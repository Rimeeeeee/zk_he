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

export default function ElectionDetail() {
  const { id } = useParams();
  const [election, setElection] = useState<Election | null>(null);
  const [selected, setSelected] = useState<number | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [_success, setSuccess] = useState(false);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);
  const [result, setResult] = useState<string | null>(null);
  const [fetchingResult, setFetchingResult] = useState(false);

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
      console.log("Voted:", selected);

      setSuccess(true);
      setSelected(null);
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
    try {
      const res = await axios.get(`http://localhost:8080/elections/${id}/result`);
      if (res.data.winner_label) {
        setResult(`🏆 Winner: ${res.data.winner_label}`);
      } else if (res.data.message) {
        setResult(res.data.message);
      } else {
        setResult("Result not available yet.");
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
          {new Date(election.start_time * 1000).toLocaleString()} →{" "}
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

          {result && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              className="text-green-400 text-center pt-3 font-medium"
            >
              {result}
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
