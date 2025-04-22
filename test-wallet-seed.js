import { CashuMint, CashuWallet } from "@cashu/cashu-ts";

async function testWalletInitialization() {
  // Create a simple test seed
  const seedString = "test seed for deterministic messages";
  
  // Convert the seed string to a buffer as we did in the fix
  const seedBuffer = new TextEncoder().encode(seedString);
  
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
    
    // Initialize the wallet with our properly encoded seed
    const wallet = new CashuWallet(mint, {
      unit: "xsr",
      keys: {
        id: "test-keyset",
        unit: "xsr",
        keys: {}
      },
      seed: seedBuffer
    });
    
    console.log("✅ Wallet initialized successfully with encoded seed");
    console.log("Wallet instance:", wallet);
    
    // Now let's try with a string seed (this should fail)
    try {
      const walletWithStringSeed = new CashuWallet(mint, {
        unit: "xsr",
        keys: {
          id: "test-keyset",
          unit: "xsr",
          keys: {}
        },
        seed: seedString // Passing the string directly
      });
      
      console.log("❌ Expected failure with string seed, but got success");
    } catch (error) {
      console.log("✅ As expected, string seed failed:", error.message);
    }
    
    return "Test completed successfully";
  } catch (error) {
    console.error("❌ Test failed:", error);
    return `Test failed: ${error.message}`;
  }
}

// Run the test
testWalletInitialization().then(console.log);