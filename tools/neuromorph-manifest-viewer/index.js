const fs = require("fs");
const path = require("path");

/**
 * Load the latest neuromorph manifest and return kernels that are
 * approved for a given AutonomyLevel and backend (e.g., "OrganicCPU").
 */
function loadNeuromorphKernels(repoRoot, maxAutonomyLevel, backend) {
  const manifestsDir = path.join(repoRoot, "research-manifests");
  const files = fs.readdirSync(manifestsDir)
    .filter(f => f.includes("neuromorph-manifest") && f.endsWith(".json"))
    .sort();

  if (files.length === 0) {
    throw new Error("No neuromorph manifests found");
  }

  const latest = files[files.length - 1];
  const raw = fs.readFileSync(path.join(manifestsDir, latest), "utf8");
  const manifest = JSON.parse(raw);

  return manifest.kernels.filter(k => {
    const level = k.autonomy_level;
    const levelRank = level === "GlobalMesh" ? 3 :
                      level === "CityMesh" ? 2 :
                      level === "LabOnly" ? 1 : 0;
    const maxRank = maxAutonomyLevel === "GlobalMesh" ? 3 :
                    maxAutonomyLevel === "CityMesh" ? 2 :
                    maxAutonomyLevel === "LabOnly" ? 1 : 0;
    const allowed = levelRank <= maxRank;
    const backendOk = (k.permitted_backends || []).includes(backend);
    return allowed && backendOk;
  });
}

module.exports = { loadNeuromorphKernels };
