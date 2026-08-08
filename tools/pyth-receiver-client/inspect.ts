import {
  DEFAULT_RECEIVER_PROGRAM_ID,
  DEFAULT_WORMHOLE_PROGRAM_ID,
  PythSolanaReceiver,
  PythTransactionBuilder,
} from "@pythnetwork/pyth-solana-receiver";

console.log(
  JSON.stringify(
    {
      receiverProgramId:
        DEFAULT_RECEIVER_PROGRAM_ID.toBase58(),
      wormholeProgramId:
        DEFAULT_WORMHOLE_PROGRAM_ID.toBase58(),
      receiverMethods:
        Object.getOwnPropertyNames(
          PythSolanaReceiver.prototype,
        ).sort(),
      transactionBuilderMethods:
        Object.getOwnPropertyNames(
          PythTransactionBuilder.prototype,
        ).sort(),
    },
    null,
    2,
  ),
);
