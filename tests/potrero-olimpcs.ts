import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PotreroOlimpcs } from "../target/types/potrero_olimpcs";
import { PublicKey, SystemProgram } from "@solana/web3.js"
import * as web3 from "@solana/web3.js"

describe("olimpics", () => {
  const name = "Test1"
  const endTime = new anchor.BN(Date.now() + 1_000_000)
  const open_until_time = new anchor.BN(Date.now() + 100_000)

  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.PotreroOlimpcs as Program<PotreroOlimpcs>;
  const maker = (program.provider as anchor.AnchorProvider).wallet;

  const [oracleEvent] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("OracleEvent"), Buffer.from(name)],
    program.programId,
  )

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize(
      name,
      endTime,
      open_until_time
    )
      .accounts({
        maker: maker.publicKey,
        oracleEvent,
        resolver: maker.publicKey
      })
      .rpc({skipPreflight: true}).catch((e)=>console.log(e));
    console.log("Your transaction signature", tx);
    const games = await program.account.podiumPrediction.all()
    console.log(games)
  });
});