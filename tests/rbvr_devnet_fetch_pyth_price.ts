import { HermesClient } from "@pythnetwork/hermes-client";
import { assert } from "chai";

const PYTH_USDC_USD_FEED_ID =
  "eaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a";

async function main(): Promise<void> {
  /*
   * The current Hermes endpoint remains usable before the
   * August 18, 2026 Pyth Core cutover.
   *
   * PYTH_API_KEY is supported when provided but is not printed.
   */
  const hermes = new HermesClient(
    "https://hermes.pyth.network",
  );

  const response = await hermes.getLatestPriceUpdates(
    [`0x${PYTH_USDC_USD_FEED_ID}`],
    {
      encoding: "base64",
      parsed: true,
    },
  );

  assert.isArray(response.binary.data);
  assert.isAbove(
    response.binary.data.length,
    0,
    "Hermes returned no binary price-update payload",
  );

  const parsed = response.parsed?.[0];

  assert.isDefined(
    parsed,
    "Hermes did not return a parsed USDC/USD price",
  );

  const returnedFeedId =
    parsed!.id.startsWith("0x")
      ? parsed!.id.slice(2)
      : parsed!.id;

  assert.equal(
    returnedFeedId.toLowerCase(),
    PYTH_USDC_USD_FEED_ID,
    "Hermes returned the wrong feed",
  );

  assert.isAbove(
    Number(parsed!.price.price),
    0,
    "USDC/USD price must be positive",
  );

  console.log(
    JSON.stringify(
      {
        hermesEndpoint: "https://hermes.pyth.network",
        feedId: returnedFeedId,
        price: parsed!.price.price,
        exponent: parsed!.price.expo,
        confidence: parsed!.price.conf,
        publishTime: parsed!.price.publish_time,
        binaryUpdateCount: response.binary.data.length,
        firstBinaryUpdateBytes: Buffer.from(
          response.binary.data[0],
          "base64",
        ).length,
      },
      null,
      2,
    ),
  );
}

main().catch((error: unknown) => {
  console.error(error);
  process.exitCode = 1;
});
