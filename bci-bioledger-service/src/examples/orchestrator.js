fetch("http://127.0.0.1:8081/bci/apply", {
  method: "POST",
  headers: { "content-type": "application/json" },
  body: JSON.stringify({
    identity: {
      issuer_did: "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7",
      subject_role: "system-daemon",
      network_tier: "inner-core",
      knowledge_factor: 0.9
    },
    required_knowledge_factor: 0.6,
    timestamp_utc: "2026-01-27T19:10:00Z",
    event: {
      session_id: "session-001",
      host_id: "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7",
      environment_id: "phx-lab",
      channel: "eeg",
      intent_label: "cursor-move",
      risk_score: 0.1,
      latency_budget_ms: 200,
      token_budget: 512,
      eco_cost_estimate: 50.0
    }
  })
});
