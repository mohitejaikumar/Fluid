import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Contract } from "../target/types/contract";
import { AccountMeta, AddressLookupTableAccount, AddressLookupTableProgram, ComputeBudgetProgram, Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, SYSVAR_INSTRUCTIONS_PUBKEY, SYSVAR_RENT_PUBKEY, TransactionInstruction, TransactionMessage, VersionedTransaction } from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, createAssociatedTokenAccount, getAssociatedTokenAddressSync, getOrCreateAssociatedTokenAccount, TOKEN_PROGRAM_ID, transferChecked } from "@solana/spl-token";
import { BN } from "bn.js";
import * as os from "os";
import * as path from "path";
import { assert } from "chai";

describe("contract", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.contract as Program<Contract>;
  const provider = anchor.getProvider();

  let owner: Keypair;
  let signer: Keypair;
  let usdcMint: PublicKey;
  let cusdcMint: PublicKey;
  let configPDA: PublicKey;
  let signerUSDC: PublicKey;
  let signerCUSDC: PublicKey;
  let vaultUSDC: PublicKey;

  let ownerUSDC: PublicKey;

  let configJLUSDC: PublicKey;

  // Juplend Protocol accounts

  let jupLending: PublicKey;
  let jlusdcMint: PublicKey;
  let jlLendingAdmin: PublicKey;
  let supplyTokenReserveLiquidity: PublicKey;
  let lendingSupplyPositionOnLiquidity: PublicKey;
  let rateModel: PublicKey;
  let jupVault: PublicKey;
  let liquidity: PublicKey;
  let liquidityProgram: PublicKey;
  let rewardsRateModel: PublicKey;
  let lendingProgram: PublicKey;
  let claimAccount: PublicKey;


  // Kamino Protocol accounts

  let vaultState: PublicKey;
  let tokenVault: PublicKey;
  let baseVaultAuthority: PublicKey;
  let sharesMint: PublicKey;
  let userSharesAta: PublicKey;
  let klendProgram: PublicKey;
  let sharesTokenProgram: PublicKey;
  let eventAuthority: PublicKey;
  let kaminoVaultProgram: PublicKey;
  let farmState: PublicKey;
  let farmProgram: PublicKey;
  let userState: PublicKey;
  let farmVault: PublicKey;
  let scopePrice: PublicKey;
  let farmVaultAuthority: PublicKey;
  let reserveAccount1: PublicKey;
  let reserveAccount2: PublicKey;
  let lendingMarket1: PublicKey;
  let lendingMarket2: PublicKey;

  let lookupTableAddress: PublicKey;
  let lookupTableAccount: AddressLookupTableAccount;

  let ctokenVaultReserve1: PublicKey;
  let ctokenVaultReserve2: PublicKey;
  let lendingMarketAuthority1: PublicKey;
  let lendingMarketAuthority2: PublicKey;
  let reserveLiquiditySupplyVault1: PublicKey;
  let reserveLiquiditySupplyVault2: PublicKey;
  let reserveCollateralMint1: PublicKey;
  let reserveCollateralMint2: PublicKey;
  let collateralTokenProgram1: PublicKey;
  let collateralTokenProgram2: PublicKey;

  let instructionSysvar: PublicKey;

  let jupLendingAccounts: AccountMeta[];
  let kaminoAccounts: AccountMeta[];

  // Event listeners
  let eventListeners: Array<number> = [];
  let capturedEvents: Array<any> = [];

  const setupEventListener = (eventName: "depositEvent" | "withdrawEvent" | "rebalanceEvent" | "allocationUpdateEvent" | "viewEvent") => {
    const listener = program.addEventListener(eventName, (event, slot, signature) => {
      capturedEvents.push({
        name: eventName,
        event,
        slot,
        signature
      });
    });
    eventListeners.push(listener);
    return listener;
  };

  const displayEvents = () => {
    if (capturedEvents.length === 0) {
      console.log("\nNo events captured");
      return;
    }

    console.log("\n" + "=".repeat(80));
    console.log("CAPTURED EVENTS SUMMARY");
    console.log("=".repeat(80));
    console.log(`Total Events: ${capturedEvents.length}\n`);

    capturedEvents.forEach((eventData, index) => {
      const eventNumber = `Event #${index + 1}`;
      console.log("┌" + "─".repeat(78) + "┐");
      console.log(`│ ${eventNumber.padEnd(76)} │`);
      console.log("├" + "─".repeat(78) + "┤");
      
      // Event Type
      
      console.log(`│ Type:  ${eventData.name.padEnd(66)} │`);
      
      // Slot & Signature
      console.log(`│ Slot: ${String(eventData.slot).padEnd(70)} │`);
      if (eventData.signature) {
        console.log(`│ Signature: ${String(eventData.signature).substring(0, 60)}... │`);
      }
      
      console.log("├" + "─".repeat(78) + "┤");
      console.log("│ Event Data:".padEnd(79) + "│");
      
      // Format event data based on type
      if (eventData.name === "depositEvent") {
        const evt = eventData.event;
        console.log(`│   User: ${String(evt.user).substring(0, 57)} │`);
        console.log(`│   Amount: ${String(evt.amount).padEnd(63)} │`);
        console.log(`│   cUSDC Minted: ${String(evt.cusdcMinted).padEnd(55)} │`);
      } else if (eventData.name === "withdrawEvent") {
        const evt = eventData.event;
        console.log(`│   User: ${String(evt.user).substring(0, 57)} │`);
        console.log(`│   cUSDC Burned: ${String(evt.cusdcBurned).padEnd(55)} │`);
        console.log(`│   USDC Returned: ${String(evt.usdcReturned).padEnd(54)} │`);
      } else if (eventData.name === "rebalanceEvent") {
        const evt = eventData.event;
        console.log(`│   JupLend Balance: ${String(evt.juplendBalance).padEnd(52)} │`);
        console.log(`│   Kamino Balance: ${String(evt.kaminoBalance).padEnd(53)} │`);
      } else if (eventData.name === "allocationUpdateEvent") {
        const evt = eventData.event;
        console.log(`│   JupLend BPS: ${String(evt.juplendBps).padEnd(56)} │`);
        console.log(`│   Kamino BPS: ${String(evt.kaminoBps).padEnd(57)} │`);
      } else if (eventData.name === "viewEvent") {
        const evt = eventData.event;
        console.log(`│   User: ${String(evt.user).substring(0, 57)} │`);
        console.log(`│   User Yeild: ${String(evt.userYeild).padEnd(55)} │`);
      }
      
      console.log("└" + "─".repeat(78) + "┘");
      console.log("");
    });

    console.log("=".repeat(80) + "\n");
  };

  const simulateTransaction = async (transaction: VersionedTransaction) => {
    const simulation = await provider.connection.simulateTransaction(transaction, {
          commitment: 'confirmed',
        });
        
    console.log("Simulation result:", JSON.stringify(simulation, null, 2));
        
    if (simulation.value.err) {
        console.error("Simulation error:", simulation.value.err);
        console.log("Simulation logs:");
        simulation.value.logs?.forEach((log, idx) => console.log(`  ${idx}: ${log}`));
      throw new Error(`Simulation failed: ${JSON.stringify(simulation.value.err)}`);
    }
  }

  const sendTransaction = async (transaction: VersionedTransaction) => {

    try {
      // await simulateTransaction(transaction);

      const txid = await provider.connection.sendTransaction(transaction, {
        skipPreflight: false,
        maxRetries: 3,
      });
      
      await provider.connection.confirmTransaction(txid, 'confirmed');
      console.log("Transaction confirmed");

      console.log("Your transaction signature", txid);
    } catch (error) {
      console.error("Error name:", error.name);
      console.error("Error message:", error.message);
      
      if (error.logs) {
        console.error("\nTransaction logs:");
        error.logs.forEach((log, idx) => console.error(`  ${idx}: ${log}`));
      }
      
      throw error;
    }
  }

  const buildVersionedTransaction = async (ix: TransactionInstruction): Promise<VersionedTransaction> => {

    const computeBudgetIx = ComputeBudgetProgram.setComputeUnitLimit({
      units: 1_400_000, // Increase from default 200k to 1.4M
    });

    const computePriceIx = ComputeBudgetProgram.setComputeUnitPrice({
      microLamports: 1,
    });

    const { blockhash } = await provider.connection.getLatestBlockhash();
    const messageV0 = new TransactionMessage({
      payerKey: signer.publicKey,
      recentBlockhash: blockhash,
      instructions: [computeBudgetIx, computePriceIx, ix],
    }).compileToV0Message([lookupTableAccount]);

    const transaction = new VersionedTransaction(messageV0);

    transaction.sign([signer]);

    return transaction;
  }

  const extendLookupTable = async (addressesToAdd: PublicKey[]) => {

    const BATCH_SIZE = 20;
    for (let i = 0; i < addressesToAdd.length; i += BATCH_SIZE) {
      const batch = addressesToAdd.slice(i, i + BATCH_SIZE);
      console.log(`Adding batch ${Math.floor(i / BATCH_SIZE) + 1} with ${batch.length} addresses...`);
      
      const extendInstruction = AddressLookupTableProgram.extendLookupTable({
        payer: signer.publicKey,
        authority: signer.publicKey,
        lookupTable: lookupTableAddress,
        addresses: batch,
      });

      const extendLookupTableTx = new VersionedTransaction(
        new TransactionMessage({
          payerKey: signer.publicKey,
          recentBlockhash: (await provider.connection.getLatestBlockhash()).blockhash,
          instructions: [extendInstruction],
        }).compileToV0Message()
      );
      extendLookupTableTx.sign([signer]);
      const extendTxSig = await provider.connection.sendTransaction(extendLookupTableTx);
      await provider.connection.confirmTransaction(extendTxSig, 'confirmed');
      
      // Small delay between batches
      await new Promise(resolve => setTimeout(resolve, 500));
    }

  }

  const createLookupTable = async (addressesToAdd: PublicKey[]) => {

    // Create Address Lookup Table
    const slot = await provider.connection.getSlot();

    const [lookupTableInst, lookupTableAddr] = AddressLookupTableProgram.createLookupTable({
      authority: signer.publicKey,
      payer: signer.publicKey,
      recentSlot: slot,
    });

    lookupTableAddress = lookupTableAddr;

    // Create the lookup table
    const createLookupTableTx = new VersionedTransaction(
      new TransactionMessage({
        payerKey: signer.publicKey,
        recentBlockhash: (await provider.connection.getLatestBlockhash()).blockhash,
        instructions: [lookupTableInst],
      }).compileToV0Message()
    );
    createLookupTableTx.sign([signer]);
    const createTxSig = await provider.connection.sendTransaction(createLookupTableTx);
    await provider.connection.confirmTransaction(createTxSig, 'confirmed');
    
    // Wait for confirmation
    await new Promise(resolve => setTimeout(resolve, 1000));

    await extendLookupTable(addressesToAdd);
    
    
    
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Fetch the lookup table account
    const lookupTableAccountInfo = await provider.connection.getAddressLookupTable(lookupTableAddress);
    if (lookupTableAccountInfo.value === null) {
      throw new Error("Failed to fetch lookup table account");
    }
    lookupTableAccount = lookupTableAccountInfo.value;
    console.log(`Lookup table loaded with ${lookupTableAccount.state.addresses.length} addresses`);
  }


  before(async ()=> {

    owner = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(require('fs').readFileSync(path.join(os.homedir(), '.config/solana/id.json'), "utf8"))));

    signer = Keypair.generate();

    await Promise.all([
      provider.connection.requestAirdrop(signer.publicKey, LAMPORTS_PER_SOL * 1000),
      provider.connection.requestAirdrop(provider.publicKey, LAMPORTS_PER_SOL * 1000)
    ]);

    usdcMint = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

    configPDA = PublicKey.findProgramAddressSync([Buffer.from("config")], program.programId)[0];

    cusdcMint = PublicKey.findProgramAddressSync([Buffer.from("cusdc-mint")], program.programId)[0];
    
    signerUSDC = getAssociatedTokenAddressSync(usdcMint, signer.publicKey);
    ownerUSDC = getAssociatedTokenAddressSync(usdcMint, owner.publicKey);
    signerCUSDC = getAssociatedTokenAddressSync(cusdcMint, signer.publicKey);

    vaultUSDC = getAssociatedTokenAddressSync(usdcMint, configPDA, true);

    // Juplend Protocol accounts

    jupLending = new PublicKey("2vVYHYM8VYnvZqQWpTJSj8o8DBf1wM8pVs3bsTgYZiqJ");
    jlusdcMint = new PublicKey("9BEcn9aPEmhSPbPQeFGjidRiEKki46fVQDyPpSQXPA2D");

    let temp = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      signer,
      jlusdcMint,
      configPDA,
      true
    );

    configJLUSDC = temp.address;

    jlLendingAdmin = new PublicKey("5nmGjA4s7ATzpBQXC5RNceRpaJ7pYw2wKsNBWyuSAZV6");

    supplyTokenReserveLiquidity = new PublicKey("94vK29npVbyRHXH63rRcTiSr26SFhrQTzbpNJuhQEDu");
    lendingSupplyPositionOnLiquidity = new PublicKey("Hf9gtkM4dpVBahVSzEXSVCAPpKzBsBcns3s8As3z77oF");

    rateModel = new PublicKey("5pjzT5dFTsXcwixoab1QDLvZQvpYJxJeBphkyfHGn688");
    jupVault = new PublicKey("BmkUoKMFYBxNSzWXyUjyMJjMAaVz4d8ZnxwwmhDCUXFB");
    liquidity = new PublicKey("7s1da8DduuBFqGra5bJBjpnvL5E9mGzCuMk1Qkh4or2Z");
    liquidityProgram = new PublicKey("jupeiUmn818Jg1ekPURTpr4mFo29p46vygyykFJ3wZC");
    rewardsRateModel = new PublicKey("5xSPBiD3TibamAnwHDhZABdB4z4F9dcj5PnbteroBTTd");
    lendingProgram = new PublicKey("jup3YeL8QhtSx1e253b2FDvsMNC87fDrgQZivbrndc9");
    claimAccount = new PublicKey("HN1r4VfkDn53xQQfeGDYrNuDKFdemAhZsHYRwBrFhsW");


    // Kamino Protocol accounts

    vaultState = new PublicKey("HDsayqAsDWy3QvANGqh2yNraqcD8Fnjgh73Mhb3WRS5E");
    tokenVault = new PublicKey("CKTEDx5z19CntAB9B66AxuS98S1NuCgMvfpsew7TQwi");
    baseVaultAuthority = new PublicKey("AyY6VCkHfTWdFs7SqBbu6AnCqLUhgzVHBzW3WcJu5Jc8");
    sharesMint = new PublicKey("7D8C5pDFxug58L9zkwK7bCiDg4kD4AygzbcZUmf5usHS");

    let temp2 = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      signer,
      sharesMint,
      configPDA,
      true,
      null,
      null,

    );

    userSharesAta = temp2.address;

    klendProgram = new PublicKey("KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD");
    sharesTokenProgram = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

    eventAuthority = new PublicKey("24tHwQyJJ9akVXxnvkekGfAoeUJXXS7mE6kQNioNySsK");
    kaminoVaultProgram = new PublicKey("KvauGMspG5k6rtzrqqn7WNn3oZdyKqLKwK2XWQ8FLjd");
    farmState = new PublicKey("9FVjHqduhDPMVqvu3cXiEBjU6nvxvGdCCLRwd9WpVRZj");
    farmProgram = new PublicKey("FarmsPZpWu9i7Kky8tPN37rs2TpmMrAZrC7S7vJa91Hr");

    userState = PublicKey.findProgramAddressSync([Buffer.from("user"), farmState.toBuffer(), configPDA.toBuffer()], farmProgram)[0];

    farmVault = new PublicKey("CRsf9nPkGBUT1HDytxfoYe3PBa5CZc9Nsh9a5aoBbGnb");
    scopePrice = new PublicKey("FarmsPZpWu9i7Kky8tPN37rs2TpmMrAZrC7S7vJa91Hr");

    farmVaultAuthority = new PublicKey("45AYN1NotDSbptLmfPrV31nrbZktJ6DLJKBxQbJMBxYg");

    reserveAccount1 = new PublicKey("Ga4rZytCpq1unD4DbEJ5bkHeUz9g3oh9AAFEi6vSauXp");
    ctokenVaultReserve1 = PublicKey.findProgramAddressSync([Buffer.from("ctoken_vault"), vaultState.toBuffer(), reserveAccount1.toBuffer()], kaminoVaultProgram)[0];
    lendingMarket1 = new PublicKey("DxXdAyU3kCjnyggvHmY5nAwg5cRbbmdyX3npfDMjjMek");
    lendingMarketAuthority1 = PublicKey.findProgramAddressSync([Buffer.from("lma"), lendingMarket1.toBuffer()], klendProgram)[0];
    reserveLiquiditySupplyVault1 = new PublicKey("GENey8es3EgGiNTM8H8gzA3vf98haQF8LHiYFyErjgrv");
    reserveCollateralMint1 = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    collateralTokenProgram1 = TOKEN_PROGRAM_ID;


    reserveAccount2 = new PublicKey("D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59");
    ctokenVaultReserve2 = PublicKey.findProgramAddressSync([Buffer.from("ctoken_vault"), vaultState.toBuffer(), reserveAccount2.toBuffer()], kaminoVaultProgram)[0];
    lendingMarket2 = new PublicKey("7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF");
    lendingMarketAuthority2 = PublicKey.findProgramAddressSync([Buffer.from("lma"), lendingMarket2.toBuffer()], klendProgram)[0];
    reserveLiquiditySupplyVault2 = new PublicKey("Bgq7trRgVMeq33yt235zM2onQ4bRDBsY5EWiTetF4qw6");
    reserveCollateralMint2 = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    collateralTokenProgram2 = TOKEN_PROGRAM_ID;

    instructionSysvar = SYSVAR_INSTRUCTIONS_PUBKEY;


    await createAssociatedTokenAccount(
      provider.connection,
      owner,
      usdcMint,
      signer.publicKey
    );

    await transferChecked(
      provider.connection, 
      owner,
      ownerUSDC,
      usdcMint,
      signerUSDC,
      owner,
      1000_000_000,
      6
    );


    // airdrop SOL to configPDA
    await provider.connection.requestAirdrop(configPDA, LAMPORTS_PER_SOL * 1000);
    

    // Collect all addresses that need to be in the lookup table
    const addressesToAdd = [
      // Base accounts from the deposit instruction
      signer.publicKey,
      configPDA,
      signerUSDC,
      signerCUSDC,
      vaultUSDC,
      cusdcMint,
      usdcMint,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
      SystemProgram.programId,
      SYSVAR_RENT_PUBKEY,
      // JupLend accounts
      jupLending,
      rewardsRateModel,
      jlusdcMint,
      configJLUSDC,
      jlLendingAdmin,
      supplyTokenReserveLiquidity,
      lendingSupplyPositionOnLiquidity,
      rateModel,
      jupVault,
      liquidity,
      liquidityProgram,
      claimAccount,
      lendingProgram,
      // Kamino accounts
      vaultState,
      tokenVault,
      baseVaultAuthority,
      sharesMint,
      userSharesAta,
      klendProgram,
      sharesTokenProgram,
      eventAuthority,
      kaminoVaultProgram,
      userState,
      farmState,
      farmVault,
      scopePrice,
      farmProgram,
      kaminoVaultProgram,
      farmVaultAuthority,
      instructionSysvar,

      reserveAccount1,
      ctokenVaultReserve1,
      lendingMarket1,
      lendingMarketAuthority1,
      reserveLiquiditySupplyVault1,
      reserveCollateralMint1,
      collateralTokenProgram1,

      reserveAccount2,
      ctokenVaultReserve2,
      lendingMarket2,
      lendingMarketAuthority2,
      reserveLiquiditySupplyVault2,
      reserveCollateralMint2,
      collateralTokenProgram2,
    ];

    await createLookupTable(addressesToAdd);

    

    // 13 accounts
    jupLendingAccounts = [
      {
        pubkey: jupLending,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: rewardsRateModel,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: jlusdcMint,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: configJLUSDC,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: jlLendingAdmin,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: supplyTokenReserveLiquidity,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: lendingSupplyPositionOnLiquidity,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: rateModel,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: jupVault,
        isSigner: false,
        isWritable: true
      },
      {
        pubkey: liquidity,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: liquidityProgram,
        isSigner: false,
        isWritable: true
      },
      {
        pubkey: claimAccount,
        isSigner: false,
        isWritable: true
      },
      {
        pubkey: lendingProgram,
        isSigner: false,
        isWritable: false
      }
   ]
   // 31 accounts
   kaminoAccounts = [
     {
       pubkey: vaultState,
       isSigner: false,
       isWritable: true
     },
     {
       pubkey: tokenVault,
       isSigner: false,
       isWritable: true
     },
     {
       pubkey: baseVaultAuthority,
       isSigner: false,
       isWritable: false
     },
     {
       pubkey: sharesMint,
       isSigner: false,
       isWritable: true
     },
     {
       pubkey: userSharesAta,
       isSigner: false,
       isWritable: true
     },
     {
       pubkey: klendProgram,
       isSigner: false,
       isWritable: false
     },
     {
       pubkey: sharesTokenProgram,
       isSigner: false,
       isWritable: false
     },
     {
       pubkey: eventAuthority,
       isSigner: false,
       isWritable: false
     },
     {
       pubkey: kaminoVaultProgram,
       isSigner: false,
       isWritable: false
     },
     {
       pubkey: userState,
       isSigner: false,
       isWritable: true
     },
     {
       pubkey: farmState,
       isSigner: false,
       isWritable: true
     },
     {
       pubkey: farmVault,
       isSigner: false,
       isWritable: true
     },
     {
       pubkey: scopePrice,
       isSigner: false,
       isWritable: false
     },
     {
       pubkey: farmProgram,
       isSigner: false,
       isWritable: false
     },
     {
       pubkey: kaminoVaultProgram,
       isSigner: false,
       isWritable: false
     },
     {
       pubkey: farmVaultAuthority,
       isSigner: false,
       isWritable: false
     },
     {
       pubkey: instructionSysvar,
       isSigner: false,
       isWritable: false
     },
     {
       pubkey: reserveAccount1,
       isSigner: false,
       isWritable: true
     },
     {
       pubkey: ctokenVaultReserve1,
       isSigner: false,
       isWritable: true
     },
     {
       pubkey: lendingMarket1,
       isSigner: false,
       isWritable: false
     },
     {
       pubkey: lendingMarketAuthority1,
       isSigner: false,
       isWritable: false
     },
     {
       pubkey: reserveLiquiditySupplyVault1,
       isSigner: false,
       isWritable: true
     },
     {
       pubkey: reserveCollateralMint1,
       isSigner: false,
       isWritable: true
     },
     {
       pubkey: collateralTokenProgram1,
       isSigner: false,
       isWritable: false
     },
     {
       pubkey: reserveAccount2,
       isSigner: false,
       isWritable: true
     },
     {
       pubkey: ctokenVaultReserve2,
       isSigner: false,
       isWritable: true
     },
     {
       pubkey: lendingMarket2,
       isSigner: false,
       isWritable: false
     },
     {
       pubkey: lendingMarketAuthority2,
       isSigner: false,
       isWritable: false
     },
     {
       pubkey: reserveLiquiditySupplyVault2,
       isSigner: false,
       isWritable: true
     },
     {
       pubkey: reserveCollateralMint2,
       isSigner: false,
       isWritable: true
     },
     {
       pubkey: collateralTokenProgram2,
       isSigner: false,
       isWritable: false
     }
   ]

  })
  
  before(async () => {
    // Set up event listeners for all tests
    setupEventListener("depositEvent");
    setupEventListener("withdrawEvent");
    setupEventListener("rebalanceEvent");
    setupEventListener("allocationUpdateEvent");
    setupEventListener("viewEvent");
  });
  
  after(async () => {
    // Clean up event listeners
    for (const listenerId of eventListeners) {
      await program.removeEventListener(listenerId);
    }
  });
 

  it("Is initialized!", async () => {
    const tx = await program.methods.initAggregatorConfig(10000).accountsStrict({
      authority: signer.publicKey,
      usdcMint: usdcMint,
      tokenProgram: TOKEN_PROGRAM_ID,
      cusdcMint: cusdcMint,
      vaultUsdc: vaultUSDC,
      config: configPDA,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
     })
     .signers([signer])
     .rpc({
      commitment: 'confirmed',
      skipPreflight: false
     });

    console.log("Your transaction signature", tx);
  });

  it("Deposit", async ()=> {
    const accounts = {
      user: signer.publicKey,
      config: configPDA,
      userUsdc: signerUSDC,
      userCusdc: signerCUSDC,
      vaultUsdc: vaultUSDC,
      cusdcMint: cusdcMint,
      usdcMint: usdcMint,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY
    };
    
    

    // Build the instruction using Anchor
    const depositIx = await program.methods.deposit(new BN(100_000_000))
      .accountsStrict(accounts)
      .remainingAccounts([
        ...jupLendingAccounts,
        ...kaminoAccounts
      ])
      .signers([signer])
      .instruction();

    
    const transaction = await buildVersionedTransaction(depositIx);

    await sendTransaction(transaction);

    const configJlUSDCBalance = await provider.connection.getTokenAccountBalance(new PublicKey(configJLUSDC));
    console.log(`configJlUSDCBalance: ${configJlUSDCBalance.value.amount}`);

    const userCUSDCBalance = await provider.connection.getTokenAccountBalance(new PublicKey(signerCUSDC));
    console.log(`userCUSDCBalance: ${userCUSDCBalance.value.amount}`);

    const userSharesBalance = await provider.connection.getTokenAccountBalance(new PublicKey(userSharesAta));
    console.log(`configKUSDCBalance: ${userSharesBalance.value.amount}`);

    // JlUSDCBlance > 0
    assert.isTrue(Number(configJlUSDCBalance.value.amount) > 0);
    assert.isTrue(Number(userCUSDCBalance.value.amount) > 0);
    // assert.isTrue(Number(userSharesBalance.value.amount) > 0);
  })

  it("Update strategy", async () => {
    const accounts = {
      config: configPDA,
      authority: signer.publicKey,
      vaultUsdc: vaultUSDC,
      usdcMint: usdcMint,
    }

    const updateConfigIx = await program.methods.updateStrategy(10000) // 100% allocation to Juplend
      .accountsStrict(accounts)
      .signers([signer])
      .rpc({
        commitment: 'confirmed',
        skipPreflight: false
      });

    console.log("Your transaction signature", updateConfigIx);
    await new Promise(resolve => setTimeout(resolve, 3000));
  })


  it("Rebalance", async () => {
    const accounts = {
      config: configPDA,
      authority: signer.publicKey,
      cusdcMint: cusdcMint,
      usdcMint: usdcMint,
      vaultUsdc: vaultUSDC,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
    }

    const rebalanceIx = await program.methods.rebalance()
      .accountsStrict(accounts)
      .remainingAccounts([
        ...jupLendingAccounts,
        ...kaminoAccounts
      ])
      .instruction();

    const transaction = await buildVersionedTransaction(rebalanceIx);

    await sendTransaction(transaction);
  })

  it("View", async () => {
    const accounts = {
      config: configPDA,
      authority: signer.publicKey,
      userCusdc: signerCUSDC,
      cusdcMint: cusdcMint,
    }

    const viewIx = await program.methods.view()
      .accountsStrict(accounts)
      .remainingAccounts([
        ...jupLendingAccounts,
        ...kaminoAccounts
      ])
      .signers([signer])
      .instruction();
      
      const transaction = await buildVersionedTransaction(viewIx);
      await sendTransaction(transaction);


  })

  it("Withdraw from Juplend", async () => {
    const accounts = {
      config: configPDA,
      user: signer.publicKey,
      userUsdc: signerUSDC,
      userCusdc: signerCUSDC,
      vaultUsdc: vaultUSDC,
      cusdcMint: cusdcMint,
      usdcMint: usdcMint,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
    };
    
    const userCUSDCBalance = await provider.connection.getTokenAccountBalance(new PublicKey(signerCUSDC));

    const withdrawIx = await program.methods.withdraw(new BN(userCUSDCBalance.value.amount))
      .accountsStrict(accounts)
      .remainingAccounts([
        ...jupLendingAccounts,
        ...kaminoAccounts
      ])
      .signers([signer])
      .instruction();


      const transaction = await buildVersionedTransaction(withdrawIx);

      await sendTransaction(transaction);

      const configJlUSDCBalance = await provider.connection.getTokenAccountBalance(new PublicKey(configJLUSDC));
      console.log(`configJlUSDCBalance: ${configJlUSDCBalance.value.amount}`);

      const userUSDCBalance = await provider.connection.getTokenAccountBalance(new PublicKey(signerUSDC));
      console.log(`userUSDCBalance: ${userUSDCBalance.value.amount}`);

      assert.isTrue(Number(userUSDCBalance.value.amount) > 0);

      // Wait for final events to propagate
      await new Promise(resolve => setTimeout(resolve, 3000));

      displayEvents();
  })

  
});
