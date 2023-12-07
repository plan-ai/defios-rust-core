import { PublicKey } from "@saberhq/solana-contrib";
import { BN } from "@project-serum/anchor";

const signatureVersion: number = 1;
const signingName: string = "defios.com";
const userName: string = "sunguru98";
const userPubkey: PublicKey = new PublicKey(
  "81sWMLg1EgYps3nMwyeSW1JfjKgFqkGYPP85vTnkFzRn"
);
const repositoryId = "12";
const roadmapTitle = "Test Roadmap";
const roadmapImageUrl = "https://github.com/defi-os/Issues";
const roadmapDescription = "https://github.com/defi-os/Issues";
const roadmapOutlook = { next2: {} };
const objectiveDeliverable = { tooling: {} };
const objectiveTitle = "Test Objective";
const objectiveDescription = "https://github.com/defi-os/Issues";
const objectiveStartUnix = new BN(1704067200);
const pullRequestMetadataUri = "https://github.com/defi-os/Issues";
const tokenName = "Hi!";
const tokenimage = "BRR";
const tokenMetadata =
  "https://en.wikipedia.org/wiki/File:Bonnet_macaque_(Macaca_radiata)_Photograph_By_Shantanu_Kuveskar.jpg";
const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);
const repositoryTitle = "Open source revolution";
const repositoryUri = "https://github.com/sunguru98/defios";
const objectiveId = "1";

export {
  signatureVersion,
  signingName,
  userName,
  userPubkey,
  repositoryId,
  roadmapTitle,
  roadmapOutlook,
  roadmapImageUrl,
  roadmapDescription,
  objectiveDeliverable,
  objectiveTitle,
  objectiveStartUnix,
  objectiveDescription,
  pullRequestMetadataUri,
  tokenName,
  tokenimage,
  tokenMetadata,
  TOKEN_METADATA_PROGRAM_ID,
  repositoryTitle,
  repositoryUri,
  objectiveId,
};
