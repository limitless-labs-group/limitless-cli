class Limitless < Formula
  desc "CLI for Limitless Exchange — browse markets, trade, and manage positions"
  homepage "https://github.com/limitless-labs-group/limitless-cli"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/limitless-labs-group/limitless-cli/releases/download/v#{version}/limitless-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER"
    else
      url "https://github.com/limitless-labs-group/limitless-cli/releases/download/v#{version}/limitless-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER"
    end
  end

  on_linux do
    url "https://github.com/limitless-labs-group/limitless-cli/releases/download/v#{version}/limitless-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "PLACEHOLDER"
  end

  def install
    bin.install "limitless"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/limitless --version")
  end
end
