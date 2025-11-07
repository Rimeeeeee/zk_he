import { useState } from "react";
import { motion } from "framer-motion";
import { useNavigate } from "react-router-dom";

interface TokenRecord {
  voterId: string;
  token: string;
}

export default function GetToken() {
  const [voterId, setVoterId] = useState("");
  const [record, setRecord] = useState<TokenRecord | null>(null);
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();

  const handleGenerate = async () => {
    if (!voterId.trim()) return alert("Please enter a valid Voter ID");
    setLoading(true);
    setRecord(null);

    // Dummy local “DB” simulation
    await new Promise((r) => setTimeout(r, 1000));

    const existing = localStorage.getItem(`token_${voterId}`);
    const token = existing ?? `tok_${btoa(voterId).slice(0, 10)}`;

    if (!existing) localStorage.setItem(`token_${voterId}`, token);

    setRecord({ voterId, token });
    setLoading(false);
  };

  return (
    <div className="min-h-screen bg-linear-to-b from-zinc-900 to-black text-gray-100 flex flex-col items-center justify-center relative overflow-hidden">
      <div className="absolute inset-0">
        <div className="absolute top-32 left-1/4 w-96 h-96 bg-cyan-500/10 rounded-full blur-3xl" />
        <div className="absolute bottom-32 right-1/4 w-96 h-96 bg-purple-500/10 rounded-full blur-3xl" />
      </div>

      <motion.div
        initial={{ opacity: 0, y: 30 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.7 }}
        className="z-10 flex flex-col items-center text-center space-y-6 p-6"
      >
        <h1 className="text-4xl md:text-5xl font-bold bg-clip-text text-transparent bg-linear-to-r from-cyan-400 to-purple-500">
          🎟️ Your Voter Token
        </h1>

        <p className="text-gray-400 max-w-xl">
          Enter your registered ID to get your secure, permanent voting token.
        </p>

        <div className="flex flex-col sm:flex-row gap-4">
          <input
            type="text"
            placeholder="Enter your Voter ID"
            value={voterId}
            onChange={(e) => setVoterId(e.target.value)}
            className="px-4 py-2 rounded-lg bg-zinc-800 border border-zinc-700 text-gray-100 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-cyan-500"
          />
          <button
            onClick={handleGenerate}
            disabled={loading}
            className="px-6 py-2 rounded-lg bg-cyan-600 hover:bg-cyan-500 text-white font-semibold shadow-lg shadow-cyan-500/30 transition-transform hover:scale-105"
          >
            {loading ? "Processing..." : "Generate / Fetch Token"}
          </button>
        </div>

        {record && (
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            className="flex flex-col items-center space-y-4"
          >
            <div className="bg-zinc-800 border border-zinc-700 rounded-xl px-6 py-4 font-mono text-cyan-400 shadow-inner">
              {record.token}
            </div>
            <p className="text-gray-500 text-sm">
              Linked to Voter ID:{" "}
              <span className="text-purple-400 font-medium">{record.voterId}</span>
            </p>
            <button
              onClick={() => navigate("/election/1")}
              className="px-6 py-2 bg-purple-700 hover:bg-purple-600 rounded-lg text-white shadow-md transition-transform hover:scale-105"
            >
              Proceed to Vote →
            </button>
          </motion.div>
        )}

        <div className="pt-10 text-gray-500 text-sm">
          © {new Date().getFullYear()} SecureVote Labs
        </div>
      </motion.div>
    </div>
  );
}
