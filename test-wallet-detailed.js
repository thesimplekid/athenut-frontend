import { CashuMint, CashuWallet } from "@cashu/cashu-ts";

async function testDetailedWalletInit() {
  console.log('Starting detailed wallet initialization test...');
  
  // Create a test seed
  const seedString = "test seed for deterministic messages 12345";
  
  try {
    // Convert the seed string to a Uint8Array as we do in the app
    const seedBuffer = new TextEncoder().encode(seedString);
    console.log('Seed buffer type:', seedBuffer.constructor.name);
    console.log('Seed buffer length:', seedBuffer.length);
    
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
    
    // Test with bip39seed
    try {
      console.log('\nTesting with bip39seed (correct according to type definition)...');
      const wallet1 = new CashuWallet(mint, {
        unit: "xsr",
        keys: {
          id: "test-keyset",
          unit: "xsr",
          keys: {}
        },
        bip39seed: seedBuffer
      });
      
      console.log('Wallet initialized with bip39seed:', wallet1);
      console.log('Internal seed value from wallet:', wallet1._seed);
      
      // Monkey patch to test if the seed was set properly
      const originalCreateOutputData = wallet1.createOutputData;
      wallet1.createOutputData = function(...args) {
        console.log('createOutputData called with args:', args);
        console.log('Internal seed at call time:', this._seed);
        return originalCreateOutputData.apply(this, args);
      };
      
      try {
        // Try to call a method that would use the seed
        console.log('Attempting to create output data...');
        wallet1.createOutputData([1], 0);
      } catch (error) {
        console.log('Error calling createOutputData:', error.message);
      }
    } catch (error) {
      console.error('Error with bip39seed:', error);
    }
    
    // Try with alternative property names
    try {
      console.log('\nTesting with seed (not in type definition)...');
      const wallet2 = new CashuWallet(mint, {
        unit: "xsr",
        keys: {
          id: "test-keyset",
          unit: "xsr",
          keys: {}
        },
        seed: seedBuffer
      });
      
      console.log('Wallet initialized with seed:', wallet2);
    } catch (error) {
      console.error('Error with seed:', error);
    }
    
    // Try with no seed
    try {
      console.log('\nTesting with no seed...');
      const wallet3 = new CashuWallet(mint, {
        unit: "xsr",
        keys: {
          id: "test-keyset",
          unit: "xsr",
          keys: {}
        }
      });
      
      console.log('Wallet initialized with no seed:', wallet3);
    } catch (error) {
      console.error('Error with no seed:', error);
    }
    
    // Try with string seed
    try {
      console.log('\nTesting with string seed...');
      const wallet4 = new CashuWallet(mint, {
        unit: "xsr",
        keys: {
          id: "test-keyset",
          unit: "xsr",
          keys: {}
        },
        bip39seed: seedString
      });
      
      console.log('Wallet initialized with string seed:', wallet4);
    } catch (error) {
      console.error('Error with string seed:', error);
    }
    
    return "Test completed successfully";
  } catch (error) {
    console.error('Main test failed:', error);
    return `Test failed: ${error.message}`;
  }
}

// Run the test
testDetailedWalletInit().then(console.log);