import { NextRequest, NextResponse } from 'next/server';

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    
    // Forward request to Rust backend with query parameters
    const url = new URL('http://localhost:8080/proof-verify');
    url.searchParams.append('proof', body.proof);
    url.searchParams.append('public_inputs', body.public_inputs);

    const response = await fetch(url.toString(), {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Verification failed' }));
      return NextResponse.json(errorData, { status: response.status });
    }

    const data = await response.json();
    return NextResponse.json(data);
  } catch (error) {
    console.error('Proof verify API error:', error);
    return NextResponse.json(
      { error: 'Failed to verify proof', details: error instanceof Error ? error.message : 'Unknown error' },
      { status: 500 }
    );
  }
}