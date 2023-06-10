const PROGRAM_ID = '7aDYtX4L9sRykPoo5mGAoKfDgjVMcWoo3D6B5AiUa99F';

const path = require('path');
const programDir = path.join(__dirname, 'programs','defios');
const idlDir = path.join(__dirname,'target','idl');
const sdkDir = path.join(__dirname,'src');
const PROGRAM_NAME = "defios";
const binaryInstallDir = path.join(__dirname,'target');

module.exports = {
  idlGenerator: 'anchor',
  programName: PROGRAM_NAME,
  programId: PROGRAM_ID,
  idlDir,
  sdkDir,
  binaryInstallDir,
  programDir,
};
