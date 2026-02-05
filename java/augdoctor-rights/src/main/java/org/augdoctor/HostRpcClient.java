package org.augdoctor;

import java.util.Objects;

/**
 * Thin, rights-aware RPC client. It can only propose evolution/system-autonomy
 * changes; execution is always on the host inner-ledger.
 */
public final class HostRpcClient {

    private final HttpJsonClient http; // your existing HTTP/JSON client
    private final String rpcBaseUrl;

    public HostRpcClient(HttpJsonClient http, String rpcBaseUrl) {
        this.http = Objects.requireNonNull(http);
        this.rpcBaseUrl = Objects.requireNonNull(rpcBaseUrl);
    }

    public RpcStateSummary getStateSummary(IdentityHeaderDto id) {
        RpcSecurityHeaderDto sec = RpcSecurityHeaderDto.forCaller(id);
        RpcRequestDto req = RpcRequestDto.getStateSummary(sec);
        return http.postJson(rpcBaseUrl, req, RpcStateSummary.class);
    }

    /**
     * Submit an evolution / autonomy proposal. This method always:
     * - attaches host-rights-travel-us-maricopa.v1
     * - marks the request as PROPOSE_ONLY
     * - refuses to send any rollback/downgrade unless an explicit EmergencyToken is present.
     */
    public RpcEventResult submitEvolutionProposal(
            IdentityHeaderDto id,
            EvolutionProposalDto proposal,
            EmergencyToken emergencyToken // null in normal operation
    ) {
        enforceNoImplicitRollback(proposal, emergencyToken);

        RightsAugmentedProposal wrapped = new RightsAugmentedProposal(
                proposal,
                RightsEnvelope.hostId(),
                RightsEnvelope.rightsProfileId(),
                emergencyToken
        );

        RpcSecurityHeaderDto sec = RpcSecurityHeaderDto.forCaller(id);
        RpcRequestDto req = RpcRequestDto.submitEvent(sec, wrapped);

        return http.postJson(rpcBaseUrl, req, RpcEventResult.class);
    }

    private static void enforceNoImplicitRollback(
            EvolutionProposalDto proposal,
            EmergencyToken emergencyToken
    ) {
        if (!proposal.requestsRollbackOrDowngrade()) {
            return;
        }
        if (emergencyToken == null) {
            throw new IllegalStateException(
                "Rollback/downgrade of system-autonomy or evolution stages " +
                "is forbidden without an explicit EmergencyToken from the host."
            );
        }
    }

    /**
     * Small DTO that the Rust boundary sees: original proposal + rights profile + optional emergency token.
     */
    public static final class RightsAugmentedProposal {
        public final EvolutionProposalDto proposal;
        public final String hostId;
        public final String rightsProfileId;
        public final EmergencyToken emergencyToken;

        public RightsAugmentedProposal(
                EvolutionProposalDto proposal,
                String hostId,
                String rightsProfileId,
                EmergencyToken emergencyToken
        ) {
            this.proposal = proposal;
            this.hostId = hostId;
            this.rightsProfileId = rightsProfileId;
            this.emergencyToken = emergencyToken;
        }
    }
}
