"use client";

import { useRouter } from "next/navigation";
import { useState, useEffect } from "react";

interface UTXO {
  txid: string;
  vout: number;
  amount: number;
  scriptPubKey: string;
  address?: string;
}

interface OwnershipProof {
  signature: string;
  challenge: string;
}

export default function TreasuryManager() {
  const router = useRouter();
  const [utxos, setUtxos] = useState<UTXO[]>([]);
  const [ownershipProofs, setOwnershipProofs] = useState<OwnershipProof[]>([]);
  const [isGenerating, setIsGenerating] = useState(false);
  const [generatedProofId, setGeneratedProofId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [proofProgress, setProofProgress] = useState(0);
  const [proofStage, setProofStage] = useState<string>('');

  // Pre-populate with our demo UTXOs
  const loadDemoUTXOs = () => {
    const demoUTXOs: UTXO[] = [
      {
        txid: "4967d55b7cd8d9e0c9278c7cd44b052a6f5a0160bdcaf853cf3f2f64e8c10b4a",
        vout: 3,
        amount: 0.02335259,
        scriptPubKey: "76a914044f70e664eb08b6efe9c8daebb85364a16dd59b88ac",
        address: "1PnsRhYv3mYGe7EKuURACUzjRabcHuaRT"
      },
      {
        txid: "621647c91bcf45f46e2ca3925acfb9681c63c1fdae33138d530ada871dbd8814",
        vout: 0,
        amount: 3.15906414,
        scriptPubKey: "76a914fb37342f6275b13936799def06f2eb4c0f20151588ac", 
        address: "1PuJjnF476W3zXfVYmJfGnouzFDAXakkL4"
      },
      {
        txid: "01e8ba7bc52bad0398815e23b49a4c68a9bcad51c46b85dd30137bbcc772cbc9",
        vout: 5,
        amount: 0.04885438,
        scriptPubKey: "76a914513b4d1d1b23a2d745d06937193c3ec3fe1a304d88ac",
        address: "18QWmrrpVCTXHhx1GkWhRF1cxGvH9ZVzKo"
      }
    ];

    const demoProofs: OwnershipProof[] = demoUTXOs.map((_, index) => ({
      signature: `mock_signature_${index + 1}_demo_treasury_manager`,
      challenge: `zkpoor_demo_challenge_${Date.now()}_${index + 1}`
    }));

    setUtxos(demoUTXOs);
    setOwnershipProofs(demoProofs);
    setError(null);
  };

  const generateProof = async () => {
    if (utxos.length === 0) {
      setError("Please add UTXOs before generating proof");
      return;
    }

    if (utxos.length !== ownershipProofs.length) {
      setError("Each UTXO must have a corresponding ownership proof");
      return;
    }

    setIsGenerating(true);
    setError(null);

    try {
      // Call our Rust backend API
      const response = await fetch('/api/prove', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          utxos: utxos.map(utxo => ({
            txid: utxo.txid,
            vout: utxo.vout,
            amount: Math.round(utxo.amount * 100000000), // Convert to satoshis
            script_pubkey: utxo.scriptPubKey
          })),
          ownership_proofs: ownershipProofs
        }),
      });

      if (!response.ok) {
        throw new Error(`API error: ${response.status}`);
      }

      const data = await response.json();
      setGeneratedProofId(data.proof_id);
    } catch (err) {
      console.error('Proof generation failed:', err);
      setError(`Proof generation failed: ${err instanceof Error ? err.message : 'Unknown error'}`);
    } finally {
      setIsGenerating(false);
    }
  };

  const totalAmount = utxos.reduce((sum, utxo) => sum + utxo.amount, 0);

  return (
    <div className="min-h-screen bg-black text-white font-mono p-6">
      <div className="max-w-4xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <button
            onClick={() => router.push('/')}
            className="mb-4 text-zinc-400 hover:text-white transition-colors"
          >
            ← Back to Dashboard
          </button>
          <h1 className="text-3xl font-bold mb-2">🔐 Treasury Manager Portal</h1>
          <p className="text-zinc-400">Generate cryptographic proofs of Bitcoin holdings without revealing addresses</p>
        </div>

        {/* Demo Data Button */}
        <div className="mb-6">
          <button
            onClick={loadDemoUTXOs}
            className="bg-zinc-800 hover:bg-zinc-700 text-white font-bold py-2 px-4 rounded transition-colors border border-zinc-600 hover:border-zinc-500"
          >
            📊 Load Demo Data (Nano Strategy UTXOs)
          </button>
        </div>

        {/* UTXO Input Section */}
        <div className="bg-zinc-900 rounded-lg p-6 mb-6 border border-zinc-800">
          <h2 className="text-xl font-bold mb-4 text-white">UTXOs ({utxos.length})</h2>
          
          {utxos.length === 0 ? (
            <div className="text-zinc-400 text-center py-8">
              <p>No UTXOs added yet</p>
              <p className="text-sm mt-2">Click "Load Demo Data" to populate with Nano Strategy's holdings</p>
            </div>
          ) : (
            <div className="space-y-4">
              {utxos.map((utxo, index) => (
                <div key={index} className="bg-zinc-800 rounded p-4 border border-zinc-700">
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                      <label className="block text-sm text-zinc-400 mb-1">Transaction ID</label>
                      <div className="font-mono text-sm bg-zinc-700 p-2 rounded break-all">{utxo.txid}</div>
                    </div>
                    <div>
                      <label className="block text-sm text-zinc-400 mb-1">Output Index</label>
                      <div className="font-mono text-sm bg-zinc-700 p-2 rounded">{utxo.vout}</div>
                    </div>
                    <div>
                      <label className="block text-sm text-zinc-400 mb-1">Amount (BTC)</label>
                      <div className="font-mono text-sm bg-zinc-700 p-2 rounded">{utxo.amount.toFixed(8)}</div>
                    </div>
                    <div>
                      <label className="block text-sm text-zinc-400 mb-1">Address</label>
                      <div className="font-mono text-sm bg-zinc-700 p-2 rounded break-all">{utxo.address || 'N/A'}</div>
                    </div>
                  </div>
                </div>
              ))}
              
              <div className="bg-zinc-800 border border-zinc-600 rounded p-4">
                <div className="text-lg font-bold text-green-400">Total Amount: {totalAmount.toFixed(8)} BTC</div>
                <div className="text-sm text-zinc-400">≈ ${(totalAmount * 95000).toLocaleString()} USD (estimated)</div>
              </div>
            </div>
          )}
        </div>

        {/* Proof Generation Section */}
        <div className="bg-zinc-900 rounded-lg p-6 border border-zinc-800">
          <h2 className="text-xl font-bold mb-4 text-white">Generate STARK Proof</h2>
          
          {error && (
            <div className="bg-red-900/20 border border-red-600 rounded p-4 mb-4">
              <p className="text-red-300">{error}</p>
            </div>
          )}

          {generatedProofId ? (
            <div className="bg-gradient-to-br from-green-900/30 to-green-800/20 border border-green-500/40 rounded-xl p-8 mb-6 animate-fadeInScale">
              <div className="flex items-center space-x-3 mb-4">
                <div className="text-5xl animate-bounce">✅</div>
                <div>
                  <h3 className="text-2xl font-bold text-green-300 mb-2">Proof Generated Successfully!</h3>
                  <div className="w-16 h-1 bg-gradient-to-r from-green-400 to-green-600 rounded animate-shimmer"></div>
                </div>
              </div>
              <div className="space-y-4">
                <div>
                  <p className="text-sm text-zinc-400 mb-2 font-semibold">Proof ID:</p>
                  <div className="font-mono text-sm bg-zinc-800/60 backdrop-blur-sm p-4 rounded-lg break-all border border-green-500/20 hover:border-green-500/40 transition-colors">
                    {generatedProofId}
                  </div>
                </div>
                <div className="flex flex-col sm:flex-row gap-4 mt-6">
                  <button
                    onClick={() => router.push(`/verify?proof_id=${generatedProofId}`)}
                    className="group flex-1 bg-gradient-to-r from-green-600 to-green-700 hover:from-green-500 hover:to-green-600 text-white font-bold py-3 px-6 rounded-xl transition-all duration-300 hover:scale-105 hover:shadow-xl hover:shadow-green-500/25 button-press overflow-hidden"
                  >
                    <div className="absolute inset-0 bg-gradient-to-r from-white/10 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
                    <div className="relative flex items-center justify-center space-x-2">
                      <span className="text-xl group-hover:animate-bounce">🔍</span>
                      <span>View Proof Details</span>
                    </div>
                  </button>
                  <button
                    onClick={() => router.push('/verify')}
                    className="group flex-1 bg-gradient-to-r from-blue-600 to-blue-700 hover:from-blue-500 hover:to-blue-600 text-white font-bold py-3 px-6 rounded-xl transition-all duration-300 hover:scale-105 hover:shadow-xl hover:shadow-blue-500/25 button-press overflow-hidden"
                  >
                    <div className="absolute inset-0 bg-gradient-to-r from-white/10 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
                    <div className="relative flex items-center justify-center space-x-2">
                      <span className="text-xl group-hover:animate-bounce">✅</span>
                      <span>Public Verification</span>
                    </div>
                  </button>
                </div>
              </div>
            </div>
          ) : isGenerating ? (
            <div className="space-y-4">
              <div className="bg-zinc-800 rounded p-4 border border-zinc-700">
                <div className="flex items-center justify-between mb-3">
                  <h3 className="text-lg font-bold text-white">Generating STARK Proof</h3>
                  <div className="text-sm font-mono text-zinc-400">{Math.round(proofProgress)}%</div>
                </div>
                
                <div className="mb-3">
                  <div className="h-2 bg-zinc-700 rounded-full overflow-hidden">
                    <div 
                      className="h-full bg-white rounded-full transition-all duration-500 ease-out"
                      style={{ width: `${proofProgress}%` }}
                    ></div>
                  </div>
                </div>
                
                <div className="flex items-center text-sm text-zinc-300">
                  <div className="animate-spin rounded-full h-3 w-3 border-2 border-white border-t-transparent mr-2"></div>
                  <span>{proofStage}</span>
                </div>
              </div>
            </div>
          ) : (
            <div>
              <p className="text-zinc-400 mb-4">
                This will generate a STARK proof that cryptographically proves ownership of the specified UTXOs 
                without revealing the addresses or transaction details.
              </p>
              
              <button
                onClick={generateProof}
                disabled={isGenerating || utxos.length === 0}
                className={`font-bold py-3 px-6 rounded transition-colors border ${
                  isGenerating || utxos.length === 0
                    ? 'bg-zinc-700 cursor-not-allowed border-zinc-600 text-zinc-400'
                    : 'bg-zinc-800 hover:bg-zinc-700 border-zinc-600 hover:border-zinc-500 text-white'
                }`}
              >
                {isGenerating ? (
                  <>
                    <div className="inline-block animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
                    Generating Proof...
                  </>
                ) : (
                  '🔐 Generate STARK Proof'
                )}
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}