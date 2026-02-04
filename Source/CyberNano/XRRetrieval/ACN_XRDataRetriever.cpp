#include "XRRetrieval/CN_XRDataRetriever.h"
#include "CyberNanoRustBridge.h"

bool UCN_XRDataRetriever::PullXRHostSnapshot(
    const FString& HostID,
    FXRHostSnapshot& OutSnapshot)
{
    FCyberNanoXRRequest Req;
    Req.Action = ECyberNanoXRAction::HostSnapshot;
    Req.HostId = HostID;

    FCyberNanoXRResponse Res;
    if (!UCyberNanoRustBridge::SendXRRequest(Req, Res))
    {
        return false;
    }
    if (!Res.bCompliant)
    {
        // degraded mode: do not display unsafe, optionally show simulated data
        return false;
    }

    OutSnapshot = Res.HostSnapshot;
    return true;
}

bool UCN_XRDataRetriever::PullXRCorridorEnvelope(
    const FString& SessionID,
    FXRSessionEnvelope& OutEnvelope)
{
    FCyberNanoXRRequest Req;
    Req.Action = ECyberNanoXRAction::CorridorEnvelope;
    Req.SessionId = SessionID;

    FCyberNanoXRResponse Res;
    if (!UCyberNanoRustBridge::SendXRRequest(Req, Res))
    {
        return false;
    }
    if (!Res.bCompliant)
    {
        return false;
    }

    OutEnvelope = Res.SessionEnvelope;
    return true;
}
