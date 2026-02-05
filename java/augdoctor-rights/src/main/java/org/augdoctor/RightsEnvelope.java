package org.augdoctor;

public final class RightsEnvelope {
    public static final String HOST_ID =
            "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7";
    public static final String RIGHTS_PROFILE_ID =
            "host-rights-travel-us-maricopa.v1";

    private RightsEnvelope() {}

    public static String hostId() {
        return HOST_ID;
    }

    public static String rightsProfileId() {
        return RIGHTS_PROFILE_ID;
    }
}
