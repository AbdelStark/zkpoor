# zkpoor (Zero Knowledge Proof Of Outstanding Reserves)

[Live demo](https://zkpoor.starkwarebitcoin.dev) - [Slides](https://zkpoor-slides.starkwarebitcoin.dev)

**zkpoor** enables companies to prove their Bitcoin holdings cryptographically without revealing addresses or UTXOs, using STARK proofs.

Built on top of the [Bitcoin Treasury](https://github.com/block/bitcoin-treasury) foundation, zkpoor extends the original dashboard with zero-knowledge proof generation and verification capabilities.

## 🎯 Problem & Solution

**Problem:** Companies like MicroStrategy claim large BTC holdings but won't publish addresses due to security concerns. Current "proof of reserves" relies on voluntary claims without cryptographic verification.

**Solution:** zkpoor provides cryptographic proof of BTC reserves without revealing individual addresses or UTXOs, strengthening Bitcoin's culture of verifiable trust.

## 🏗️ Architecture

This is a unified web application with two main components:

1. **Enhanced Public Dashboard** - Original Bitcoin Treasury features plus "Verify with STARK" cryptographic verification
2. **Treasury Manager Portal** - Protected interface for inputting UTXOs and generating proofs

## 🚀 Development Phases

- **Phase 1:** UI/UX implementation with mocked proof generation and verification
- **Phase 2:** Backend integration with actual STARK proof generation  
- **Phase 3:** Cairo program implementation for cryptographic proofs

## Getting Started

This project is a **Next.js** application. Follow the instructions below to run the application locally.

### Prerequisites

- [Node.js](https://nodejs.org/) (ensure you have version 16 or later)
- npm (comes with Node.js) or yarn (optional package manager)

### Installation

1. Clone the repository if you haven't already:

   ```bash
   git clone https://github.com/AbdelStark/zkpoor.git
   cd zkpoor
   ```

2. Install dependencies:
   ```bash
   npm install
   ```
   Or, if you're using Yarn:
   ```bash
   yarn install
   ```

### Running the Development Server

To start the development server, run:

```bash
npm run dev
```

Or, for Yarn users:

```bash
yarn dev
```

This will start the Next.js development server on the default port (3000). Open your browser and visit:

```
http://localhost:3000
```

### Building for Production

To build the application for production, run:

```bash
npm run build
```

Or, with Yarn:

```bash
yarn build
```

This will create an optimized production build of the app in the `.next` directory. To serve it, use:

```bash
npm start
```

Or:

```bash
yarn start
```

### Available Scripts

- `npm run dev` - Starts the development server.
- `npm run build` - Builds the app for production.
- `npm start` - Runs the production server after building.

### Stopping the Server

To stop either the development or production server, press `Ctrl+C` in the terminal where the server is running.

## Roadmap ahead

- [ ] Complete the end to end proving pipeline and verification flow
- [ ] Add proving of ownership of UTXOs via signatures within the Cairo program
- [ ] Optimize the Cairo program for performance and scalability
  

## Open questions

- How to prevent someone to take a proof and reuse it later on to claim they own the same UTXOs?

Potential solution would be that the manager add as additional input a public key, that will be also outputted in the proof, and then the verifier can ask the manager to sign a message with the corresponding private key to prove he originally generated the proof.