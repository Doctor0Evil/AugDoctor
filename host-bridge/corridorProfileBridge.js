// Tiny JS bridge: load ALN (pre-rendered as JSON) and ask Rust for CorridorProfile.

import fs from "node:fs";
import path from "node:path";
import { deriveCorridorProfileFromDoctrine } from "biophysical-corridor-mutation-node"; 
// assume this is a napi-rs or wasm-bindgen wrapper around the Rust function

const HOME = process.env.HOME || process.env.USERPROFILE;

/**
 * Load evolutionturnpolicy.aln.json and deep-domain-rights.aln.json,
 * then derive a CorridorProfileBundle via Rust.
 */
export function loadCorridorProfileBundle(hostDid) {
  const evoPath = path.join(
    HOME,
    ".Organichain",
    "NeuroPC",
    "Evolution",
    "evolutionturnpolicy.aln.json"
  );

  const deepPath = path.join(
    HOME,
    ".qpudatashards",
    "deep-domain-rights.aln.json"
  );

  const evoJson = fs.readFileSync(evoPath, "utf8");
  const deepJson = fs.readFileSync(deepPath, "utf8");

  const bundle = deriveCorridorProfileFromDoctrine(evoJson, deepJson);

  if (bundle.profile.aln_profile_id === "") {
    throw new Error("Empty ALN profile id in CorridorProfile");
  }
  if (!bundle.source_deep_rights_id.includes("deep-domain-rights")) {
    throw new Error("Deep-domain rights profile id mismatch");
  }

  // You can cache this per hostDid if desired.
  return {
    hostDid,
    corridorProfile: bundle.profile,
    sourceEvoPolicyId: bundle.source_evo_policy_id,
    sourceDeepRightsId: bundle.source_deep_rights_id,
  };
}
