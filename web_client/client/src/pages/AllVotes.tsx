import { useEffect, useState } from "react";
import { motion } from "framer-motion";
import { useNavigate } from "react-router-dom";
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

export default function AllVotes() {
  const [elections, setElections] = useState<Election[]>([]);
  const [loading, setLoading] = useState(true);
  const navigate = useNavigate();

  useEffect(() => {
    axios
      .get("http://localhost:8080/elections")
      .then((res) => setElections(res.data))
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  return (
    <div className="min-h-screen bg-linear-to-b from-zinc-900 to-black text-gray-100 flex flex-col items-center justify-center px-6">
      <motion.h1
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="text-4xl font-bold mb-10 bg-clip-text text-transparent bg-linear-to-r from-cyan-400 to-purple-500"
      >
        🗳️ Active Elections
      </motion.h1>

      {loading ? (
        <div className="text-gray-400">Loading elections...</div>
      ) : elections.length === 0 ? (
        <div className="text-gray-400">No elections found.</div>
      ) : (
        <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-6 w-full max-w-6xl">
          {elections.map((election) => (
            <motion.div
              key={election.id}
              whileHover={{ scale: 1.03 }}
              className="bg-zinc-900/70 border border-zinc-800 rounded-xl p-6 shadow-lg hover:shadow-cyan-500/10 transition"
            >
              <h2 className="text-xl font-semibold text-cyan-400 mb-2">
                {election.name}
              </h2>
              <p className="text-sm text-gray-400 mb-3">
                {new Date(election.start_time * 1000).toLocaleString()} →{" "}
                {new Date(election.end_time * 1000).toLocaleString()}
              </p>
              <p className="text-gray-400 text-sm mb-4">
                {election.candidates.length} candidates
              </p>
              <button
                onClick={() => navigate(`/election/${election.id}`)}
                className="w-full py-2 rounded-lg bg-purple-700 hover:bg-purple-600 text-white font-medium transition"
              >
                View & Vote
              </button>
            </motion.div>
          ))}
        </div>
      )}
    </div>
  );
}
