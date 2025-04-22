import { CashuMint, CashuWallet } from "@cashu/cashu-ts";

async function testSeedGeneration() {
  console.log('Starting HDKey seed test...');
  
  // This is our typical seed
  const originalSeed = "my wallet seed for cashu";
  
  try {
    // First, test what happens with the raw seed
    console.log('\nTest 1: Using raw string seed...');
    const rawSeedBuffer = new TextEncoder().encode(originalSeed);
    console.log('Raw seed length:', rawSeedBuffer.length, 'bytes', rawSeedBuffer.length * 8, 'bits');
    
    try {
      // Initialize a test mint
      const mint = new CashuMint("https://example.com");
      
      // Mock the keys response
      mint.getKeys = async () => ({
        keysets: [{
          id: "test-keyset",
          unit: "xsr",
          keys: {}
        }]
      });
      
      // Try with raw seed - this should fail
      const walletRaw = new CashuWallet(mint, {
        unit: "xsr",
        keys: {
          id: "test-keyset",
          unit: "xsr",
          keys: {}
        },
        bip39seed: rawSeedBuffer
      });
      
      console.log('Raw seed worked? This should not happen!');
    } catch (error) {
      console.log('Expected error with raw seed:', error.message);
    }
    
    // Now try with proper 256-bit seed from SHA-256
    console.log('\nTest 2: Creating 256-bit seed using SHA-256...');
    
    // Convert the string to an ArrayBuffer first
    const encoder = new TextEncoder();
    const data = encoder.encode(originalSeed);
    
    // Use SHA-256 to get a 32-byte (256-bit) result
    const hashBuffer = await crypto.subtle.digest('SHA-256', data);
    const seedBuffer = new Uint8Array(hashBuffer);
    
    console.log('SHA-256 seed buffer length:', seedBuffer.length, 'bytes', seedBuffer.length * 8, 'bits');
    
    try {
      // Initialize a test mint
      const mint = new CashuMint("https://example.com");
      
      // Mock the keys response
      mint.getKeys = async () => ({
        keysets: [{
          id: "test-keyset",
          unit: "xsr",
          keys: {}
        }]
      });
      
      // Try with SHA-256 seed - this should work
      const wallet = new CashuWallet(mint, {
        unit: "xsr",
        keys: {
          id: "test-keyset",
          unit: "xsr",
          keys: {}
        },
        bip39seed: seedBuffer
      });
      
      console.log('SHA-256 seed worked! Wallet initialized without errors');
      console.log('Internal seed value from wallet:', wallet._seed ? 'Present, length ' + wallet._seed.length : 'Undefined');
      
      // Try patching the createOutputData
      try {
        console.log('\nTesting createOutputData with counter...');
        // Test if we can use the wallet with a counter for deterministic outputs
        const originalCreateOutputData = wallet.createOutputData;
        let createOutputDataCalled = false;
        
        wallet.createOutputData = function(...args) {
          console.log('createOutputData called with args:', args.slice(0, 3)); // Show first 3 args only
          console.log('Has seed:', !!this._seed);
          createOutputDataCalled = true;
          // Actually call the original method
          try {
            const result = originalCreateOutputData.apply(this, args);
            console.log('createOutputData succeeded!');
            return result;
          } catch (err) {
            console.log('createOutputData failed:', err.message);
            throw err;
          }
        };
        
        // The actual test - create an OutputData array with a counter
        // We're using a key pattern of 1 (amounts array has a single 1)
        const result = wallet.createOutputData([1], null, 0, null, null, null, null);
        console.log('Result from createOutputData with counter:', result ? 'Success' : 'Failed');
      } catch (err) {
        console.log('Error testing createOutputData:', err.message);
      }
    } catch (error) {
      console.error('Error with SHA-256 seed:', error);
    }
    
    return "HDKey seed test completed";
  } catch (error) {
    console.error('Main test failed:', error);
    return `HDKey seed test failed: ${error.message}`;
  }
}

// Run the test
testSeedGeneration().then(console.log);