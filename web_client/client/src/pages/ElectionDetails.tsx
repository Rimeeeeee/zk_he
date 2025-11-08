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
  const [success, setSuccess] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    axios
      .get(`http://localhost:8080/elections/${id}`)
      .then((res) => setElection(res.data))
      .catch(console.error);
  }, [id]);

  const handleVote = async () => {
    if (!selected || !election) {
      alert("Please select a candidate first");
      return;
    }

    const token = localStorage.getItem("voter_token");
    if (!token) {
      alert("You must get a voting token first!");
      return;
    }

    // Dummy ciphertext placeholder (you’ll replace with TFHE encryption)
    const ciphertext = `enc_vote_${selected}_${Date.now()}`;

    setSubmitting(true);
    setError("");
    setSuccess(false);

    try {
      await axios.post(`http://localhost:8080/elections/${election.id}/ballots`, {
        token,
        candidate_id: selected,
        ciphertext,
      });
      setSuccess(true);
    } catch (err: any) {
      console.error(err);
      setError("Failed to submit your vote. Please try again.");
    } finally {
      setSubmitting(false);
    }
  };

  if (!election) {
    return (
      <div className="min-h-screen flex items-center justify-center text-gray-400">
        Loading election...
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

        {election.closed ? (
          <div className="text-red-400 text-center font-medium">
            This election is closed.
          </div>
        ) : (
          <div className="space-y-4">
            {election.candidates.map((c) => (
              <div
                key={c.id}
                onClick={() => setSelected(c.id)}
                className={`px-4 py-3 rounded-lg border cursor-pointer transition ${
                  selected === c.id
                    ? "bg-cyan-600 border-cyan-500 text-white"
                    : "bg-zinc-800 border-zinc-700 hover:border-cyan-400"
                }`}
              >
                {c.name}
              </div>
            ))}

            <button
              onClick={handleVote}
              disabled={submitting}
              className="w-full mt-6 py-3 rounded-xl bg-purple-700 hover:bg-purple-600 text-white font-semibold shadow-lg shadow-purple-500/30 transition-transform hover:scale-105"
            >
              {submitting ? "Submitting..." : "Submit Vote"}
            </button>

            {success && (
              <div className="text-green-400 text-center pt-3 font-medium">
                ✅ Your encrypted vote was submitted!
              </div>
            )}

            {error && (
              <div className="text-red-400 text-center pt-3 font-medium">
                {error}
              </div>
            )}
          </div>
        )}
      </motion.div>
    </div>
  );
}
