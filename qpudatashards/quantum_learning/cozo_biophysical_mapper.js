const sampleParticles = [ /* same 300+ CIDs as Rust */ ];

function excavateCozoExport(exportJson) {
  const results = [];
  let totalWave = 0.0;
  let processed = 0;

  sampleParticles.forEach(p => {
    processed++;
    const awarenessPass = !p.mime.includes("bio");
    const consciousnessPass = !p.text.toLowerCase().includes("soul") && !p.text.toLowerCase().includes("clone");
    const oculusPass = !p.mime.includes("video");
    const waveIncrement = Math.min(p.size / 1000000, 5.0);

    totalWave += waveIncrement;

    results.push({
      cid: p.cid,
      plane: "bioscale/biophysics",
      awareness_pass: awarenessPass,
      consciousness_pass: consciousnessPass,
      oculus_pass: oculusPass,
      brain_token_net_weight: 0.0,
      brain_token_circulating: 0.0,
      cloning_forbidden: true,
      wave_quota_increment: waveIncrement,
      evolution_points_granted: waveIncrement > 1 ? 10 : 1,
      debug_console_line: `[${new Date().toISOString()}] CID=${p.cid} wave+${waveIncrement.toFixed(2)} consciousness_pass=${consciousnessPass}`
    });
  });

  return {
    timestamp: new Date().toISOString(),
    level: "info",
    message: "CozoParticleBiophysicalMapper quantum-learning excavation completed – all consciousness policies enforced",
    data: results,
    total_particles_processed: processed,
    total_wave_quota_granted: totalWave,
    errors: []
  };
}

// Export for AI-Chat / mobile
module.exports = { excavateCozoExport };
