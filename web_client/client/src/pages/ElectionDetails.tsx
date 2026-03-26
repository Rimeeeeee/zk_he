import { useEffect, useState } from "react";
import { motion } from "framer-motion";
import { useParams } from "react-router-dom";
import axios from "axios";

interface Candidate {
  id: number;
  name: string;
}

interface Election {
  id: string;
  name: string;
  start_time: number;
  end_time: number;
  closed: boolean;
  candidates: Candidate[];
}

interface TallyEntry {
  candidate_id: number;
  label: string;
  votes: number;
}

interface ProofBundle {
  system: string;
  proof_b64: string;
  verification_key_b64: string;
  public_totals: number[];
  verified: boolean;
  proof_hash: string;
}

interface ElectionResult {
  election_id: string;
  winner_label: string;
  winner_id: number;
  totals: TallyEntry[];
  ballot_count: number;
  tally_hash: string;
  generated_at: number;
  status: string;
  proof: ProofBundle;
}

export default function ElectionDetail() {
  const { id } = useParams();
  const [election, setElection] = useState<Election | null>(null);
  const [selected, setSelected] = useState<number | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [_success, setSuccess] = useState(false);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);
  const [result, setResult] = useState<ElectionResult | null>(null);
  const [resultMessage, setResultMessage] = useState<string | null>(null);
  const [fetchingResult, setFetchingResult] = useState(false);
  const [clientProofCheck, setClientProofCheck] = useState<string | null>(null);

  useEffect(() => {
    (async () => {
      try {
        const res = await axios.get(`http://localhost:8080/elections/${id}`);
        setElection(res.data);
      } catch (err) {
        console.error("Failed to fetch election:", err);
        setError("Failed to load election details.");
      } finally {
        setLoading(false);
      }
    })();
  }, [id]);

  const handleVote = async () => {
    if (!selected || !election) {
      alert("Please select a candidate first.");
      return;
    }

    const token = localStorage.getItem("voter_token");
    if (!token) {
      alert("You must get a voting token first!");
      return;
    }

    setSubmitting(true);
    setError("");
    setSuccess(false);

    try {
      await axios.post(`http://localhost:8080/elections/${election.id}/ballots`, {
        token,
        candidate_id: selected,
      });
      setSuccess(true);
      setSelected(null);
      setResult(null);
      setResultMessage(null);
      setClientProofCheck(null);
    } catch (err) {
      console.error("Error submitting vote:", err);
      setError("Failed to submit vote. Please try again.");
    } finally {
      setSubmitting(false);
    }
  };

  const handleGetResult = async () => {
    if (!id) return;
    setFetchingResult(true);
    setError("");
    setResult(null);
    setResultMessage(null);
    setClientProofCheck(null);

    try {
      const res = await axios.get(`http://localhost:8080/elections/${id}/result`);
      if (res.data.winner_label && res.data.proof) {
        const proofOk = await verifyProofChecksum(res.data.proof);
        setClientProofCheck(
          proofOk
            ? "Proof bundle checksum matched in the browser."
            : "Proof bundle checksum did not match."
        );
        setResult(res.data);
      } else if (res.data.message) {
        setResultMessage(res.data.message);
      } else {
        setError("Result not available yet.");
      }
    } catch (err) {
      console.error("Error fetching result:", err);
      setError("Failed to fetch result.");
    } finally {
      setFetchingResult(false);
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center text-gray-400">
        Loading election...
      </div>
    );
  }

  if (error || !election) {
    return (
      <div className="min-h-screen flex items-center justify-center text-red-400">
        {error || "Election not found."}
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-linear-to-b from-zinc-900 to-black text-gray-100 flex flex-col items-center justify-center px-6">
      <motion.div
        initial={{ opacity: 0, y: 40 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6 }}
        className="w-full max-w-2xl bg-zinc-900/80 border border-zinc-800 rounded-2xl shadow-lg p-8 backdrop-blur-md"
      >
        <h1 className="text-3xl font-bold text-center mb-4 bg-clip-text text-transparent bg-linear-to-r from-cyan-400 to-purple-500">
          {election.name}
        </h1>

        <p className="text-gray-400 text-center mb-6 text-sm">
          {new Date(election.start_time * 1000).toLocaleString()} to{" "}
          {new Date(election.end_time * 1000).toLocaleString()}
        </p>

        <div className="space-y-4">
          {election.candidates.length === 0 && (
            <div className="text-gray-500 text-center">No candidates found.</div>
          )}

          {election.candidates.map((c) => (
            <div
              key={c.id}
              onClick={() => setSelected(c.id)}
              className={`px-4 py-3 rounded-lg border cursor-pointer transition-all ${
                selected === c.id
                  ? "bg-cyan-600 border-cyan-500 text-white shadow-lg"
                  : "bg-zinc-800 border-zinc-700 hover:border-cyan-400"
              }`}
            >
              {c.name}
            </div>
          ))}

          <button
            onClick={handleVote}
            disabled={submitting}
            className={`w-full mt-6 py-3 rounded-xl text-white font-semibold shadow-lg transition-transform ${
              submitting
                ? "bg-purple-900 cursor-not-allowed"
                : "bg-purple-700 hover:bg-purple-600 hover:scale-105 shadow-purple-500/30"
            }`}
          >
            {submitting ? "Submitting..." : "Submit Encrypted Vote"}
          </button>

          <button
            onClick={handleGetResult}
            disabled={fetchingResult}
            className={`w-full mt-4 py-3 rounded-xl font-semibold shadow-lg transition-transform ${
              fetchingResult
                ? "bg-zinc-700 text-gray-400 cursor-not-allowed"
                : "bg-cyan-600 hover:bg-cyan-500 text-white hover:scale-105"
            }`}
          >
            {fetchingResult ? "Calculating Result..." : "Get Result"}
          </button>

          {resultMessage && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              className="text-green-400 text-center pt-3 font-medium"
            >
              {resultMessage}
            </motion.div>
          )}

          {result && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              className="mt-6 rounded-2xl border border-emerald-500/20 bg-emerald-500/10 p-5"
            >
              <div className="text-green-300 text-lg font-semibold text-center">
                Winner: {result.winner_label}
              </div>
              <div className="mt-2 text-center text-sm text-emerald-100/80">
                {result.status}
              </div>
              <div className="mt-4 grid gap-3 text-sm text-gray-200">
                <div className="rounded-xl bg-black/20 p-3">
                  Ballots counted: {result.ballot_count}
                </div>
                <div className="rounded-xl bg-black/20 p-3">
                  Generated:{" "}
                  {new Date(result.generated_at * 1000).toLocaleString()}
                </div>
                <div className="rounded-xl bg-black/20 p-3">
                  Proof system: {result.proof.system}
                </div>
                <div className="rounded-xl bg-black/20 p-3">
                  Server-side verification:{" "}
                  {result.proof.verified ? "passed" : "failed"}
                </div>
                {clientProofCheck && (
                  <div className="rounded-xl bg-black/20 p-3">
                    Browser integrity check: {clientProofCheck}
                  </div>
                )}
              </div>

              <div className="mt-5 space-y-3">
                <div className="text-sm font-semibold uppercase tracking-wide text-emerald-100/80">
                  Decrypted Totals
                </div>
                {result.totals.map((entry) => (
                  <div
                    key={entry.candidate_id}
                    className="flex items-center justify-between rounded-xl border border-white/10 bg-black/20 px-4 py-3 text-sm"
                  >
                    <span>{entry.label}</span>
                    <span className="font-semibold text-emerald-200">
                      {entry.votes} vote{entry.votes === 1 ? "" : "s"}
                    </span>
                  </div>
                ))}
              </div>

              <div className="mt-5 rounded-xl border border-white/10 bg-black/20 p-4 text-xs text-gray-300">
                <div className="font-medium text-cyan-300">Proof Hash</div>
                <div className="mt-1 break-all">{result.proof.proof_hash}</div>
                <div className="mt-3 font-medium text-cyan-300">Tally Hash</div>
                <div className="mt-1 break-all">{result.tally_hash}</div>
              </div>
            </motion.div>
          )}

          {error && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              className="text-red-400 text-center pt-3 font-medium"
            >
              {error}
            </motion.div>
          )}
        </div>
      </motion.div>
    </div>
  );
}

async function verifyProofChecksum(proof: ProofBundle): Promise<boolean> {
  const payload = new TextEncoder().encode(
    JSON.stringify({
      proof_b64: proof.proof_b64,
      verification_key_b64: proof.verification_key_b64,
      public_totals: proof.public_totals,
    })
  );

  const digest = await crypto.subtle.digest("SHA-256", payload);
  const digestHex = Array.from(new Uint8Array(digest))
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");

  return digestHex === proof.proof_hash;
}
