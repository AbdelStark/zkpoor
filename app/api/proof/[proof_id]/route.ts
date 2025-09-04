import { NextRequest, NextResponse } from 'next/server';

export async function GET(
  request: NextRequest,
  { params }: { params: { proof_id: string } }
) {
  try {
    const proofId = params.proof_id;
    
    // Forward request to Rust backend
    const response = await fetch(`http://localhost:8080/proof/${proofId}`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Proof not found' }));
      return NextResponse.json(errorData, { status: response.status });
    }

    const data = await response.json();
    return NextResponse.json(data);
  } catch (error) {
    console.error('Get proof API error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch proof', details: error instanceof Error ? error.message : 'Unknown error' },
      { status: 500 }
    );
  }
}