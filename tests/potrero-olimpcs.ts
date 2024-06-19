import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PotreroOlimpcs } from "../target/types/potrero_olimpcs";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import * as web3 from "@solana/web3.js";

describe("olimpics", () => {
  const name = "Test11";
  const endTime = new anchor.BN(Date.now() + 1_000_000);
  const open_until_time = new anchor.BN(Date.now() + 100_000);
  const seed = new anchor.BN(2);
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.PotreroOlimpcs as Program<PotreroOlimpcs>;
  const maker = (program.provider as anchor.AnchorProvider).wallet;

  const [oracleEvent] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("OracleEvent"), Buffer.from(name)],
    program.programId
  );

  const [prediction] = web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("PodiumPrediction"),
      Buffer.from(name),
      maker.publicKey.toBytes(),
      new anchor.BN(seed).toBuffer("le", 8),
    ],
    program.programId
  );

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods
      .initialize(name, endTime, open_until_time)
      .accounts({
        maker: maker.publicKey,
        oracleEvent,
        resolver: maker.publicKey,
      })
      .rpc({ skipPreflight: true })
    console.log("Your transaction signature", tx);
    const games = await program.account.oracleEvent.all();
    console.log(games);
  });

  it("Create a prediction!", async () => {
    // Add your test here.
    const tx = await program.methods
      .makePrediction(name,seed, 0, 0, 0)
      .accounts({
        player: maker.publicKey,
        oracleEvent,
        projectTreasury: new PublicKey("GtrjYbtvJ9T5oP1P64gY2yBLXcDtKERgNp5o1k6ty7Mj"),
        prediction,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc({ skipPreflight: true })
    console.log("Your transaction signature", tx);
    const games = await program.account.podiumPrediction.all();
    console.log(games);
  });
});
