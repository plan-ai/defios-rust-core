import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { TokenVesting } from "../target/types/token_vesting";

describe("defios", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Defios as Program<TokenVesting>;
  const {
    provider: { connection },
  } = program;

  const { web3 } = anchor;

  it("Registers a vesting contract!", async () => {});

  it("Adds schedules to vesting contract!", async () => {});

  it("Unlocks tokens on a vesting contract", async () => {});

  it("Changes the destination address of a vesting contract", async () => {});
});
