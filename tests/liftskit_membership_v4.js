const anchor = require("@project-serum/anchor");
const serumCmn = require("@project-serum/common");
const { SystemProgram } = anchor.web3;
const { TOKEN_PROGRAM_ID, Token } = require("@solana/spl-token");
const { assert } = require("chai");

describe("Token Tests", () => {

  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.LiftskitMembershipV4;

  // intial values
  const MINT_TOKENS = 4200000000000000; // 42M with 8dp
  const MINT_DECIMALS = 0;

  // intial accounts
  let mint = null;
  let god = null;
  let creatorAcc = anchor.web3.Keypair.generate();
  let creatorTokenAcc = null;

  it("Sets up initial Token Vault", async () => {
    // create God account
    const [_mint, _god] = await serumCmn.createMintAndVault(
      program.provider,
      new anchor.BN(MINT_TOKENS),
      undefined,
      MINT_DECIMALS
    );
    mint = _mint;
    god = _god;

    // create a User to pay
    creatorTokenAcc = await serumCmn.createTokenAccount(
      program.provider,
      mint,
      creatorAcc.publicKey
    );
  });

  it("Creates a membership list.", async () => {
    /* Call the create function via RPC */
    const baseAccount = anchor.web3.Keypair.generate();
    await program.rpc.create(
      baseAccount.publicKey,
      {
        accounts: {
          baseAccount: baseAccount.publicKey,
          user: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        },
        signers: [baseAccount],
    });

    /* Fetch the account and check the value of count */
    const account = await program.account.baseAccount.fetch(baseAccount.publicKey);
    console.log('Count 0: ', account.members)
    assert.ok(account.members.length == 1);
    _baseAccount = baseAccount;

  });

  it("Adds member to the membership list.", async () => {
    const baseAccount = _baseAccount;

  const newAccount = anchor.web3.Keypair.generate();
    await program.rpc.addMember(newAccount.publicKey, {
      accounts: {
        baseAccount: baseAccount.publicKey,
      },
    });

    const account = await program.account.baseAccount.fetch(baseAccount.publicKey);
    console.log('Count 1: ', account.members)
    assert.ok(account.members.length == 2);
  });

  it("God pays user account some tokens ", async () => {
    const INTERACTION_FEE = 5;

    await program.rpc.payUser(new anchor.BN(INTERACTION_FEE), {
      accounts: {
        from: god,
        to: creatorTokenAcc,
        owner: program.provider.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    });


    console.log('*************', {
      from: god.toBase58(),
      to: creatorTokenAcc.toBase58(),
      tokenProgram: TOKEN_PROGRAM_ID.toBase58(),
      programId: program.programId.toBase58(),
    });

    const godAccount = await serumCmn.getTokenAccount(program.provider, god);
    const toAccount = await serumCmn.getTokenAccount(program.provider, creatorTokenAcc);
    console.log('God Balance: ', godAccount.amount.toNumber());
    console.log('To Account Balance: ', toAccount.amount.toNumber());

    // verify tokens sent
    assert.ok(godAccount.amount == MINT_TOKENS - INTERACTION_FEE);
    assert.ok(toAccount.amount == INTERACTION_FEE);
  });

});
