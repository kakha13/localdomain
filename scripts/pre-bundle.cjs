const { execSync } = require("child_process");
const fs = require("fs");

if (process.platform === "darwin") {
  const identity =
    "Developer ID Application: KAKHA GIORGASHVILI (9KW3WQ579Q)";
  const binary = "resources/localdomain-daemon";

  // Check if binary exists
  if (!fs.existsSync(binary)) {
    console.warn(`Warning: ${binary} not found, skipping codesigning.`);
    process.exit(0);
  }

  console.log(`Signing ${binary} for notarization...`);
  try {
    execSync(
      `codesign --sign "${identity}" --options runtime --timestamp --force "${binary}"`,
      { stdio: "inherit" }
    );
    console.log("Daemon binary signed successfully.");
  } catch (error) {
    console.error(`Codesigning failed: ${error.message}`);
    if (process.env.CI) {
      console.warn("Running in CI environment, some codesigning features may be unavailable.");
      console.warn("This is expected if the certificate is not imported yet.");
    } else {
      throw error;
    }
  }
}
