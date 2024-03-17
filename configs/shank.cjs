const path = require("path");
const { generateIdl } = require("@metaplex-foundation/shank-js");

const idlDir = path.join(__dirname, "..", "idls");
const binaryInstallDir = path.join(__dirname, "..", ".crates");
const programDir = path.join(__dirname, "..", "programs");

generateIdl({
  generator: "shank",
  programName: "sol_x404_program",
  programId: "BTz8yJGxKm6jEZAncTCMHDs4uTFvi5sVMUjCBUwfVkUM",
  idlDir,
  binaryInstallDir,
  programDir: path.join(programDir, "sol-x404"),
});
