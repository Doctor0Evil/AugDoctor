export function decideRewardFromLog(logRecord) {
  const reasons = [];

  if (!logRecord.kernel_passed) {
    reasons.push("KERNEL_VIOLATION");
  }
  if (!logRecord.roh_invariant_passed) {
    reasons.push("ROH_INVARIANT_BROKEN");
  }
  if (!logRecord.cyberrank_invariant_passed) {
    reasons.push("CYBERRANK_DEGRADED");
  }
  if (logRecord.knowledge_factor_delta <= 0.0) {
    reasons.push("NO_KNOWLEDGE_GAIN");
  }

  return {
    session_id: logRecord.session_id,
    eligible: reasons.length === 0,
    reason_codes: reasons,
  };
}
