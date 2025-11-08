import init, { generate_keys_b64, extract_server_key_b64, encrypt_vote_u32_b64 } from "../../pkg/key.js";

export async function initWasm() {
  await init();
}

/**
 * Get or create keys for a given election.
 * Persists client keys in localStorage, and sends server key to backend.
 */
export async function getOrCreateKeys(electionId: string) {
  const existing = localStorage.getItem(`keys_${electionId}`);
  if (existing) return existing;

  const keysB64 = await generate_keys_b64();
  localStorage.setItem(`keys_${electionId}`, keysB64);

  const serverKeyB64 = await extract_server_key_b64(keysB64);

  await fetch(`http://localhost:8080/elections/${electionId}/server-key`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ server_key: serverKeyB64 }),
  });

  return keysB64;
}

export async function encryptVote(keysB64: string, candidateId: number) {
  return await encrypt_vote_u32_b64(keysB64, candidateId);
}
