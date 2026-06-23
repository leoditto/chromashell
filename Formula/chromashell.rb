class Chromashell < Formula
  desc "Live shader effects for your terminal"
  homepage "https://github.com/leoditto/chromashell"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/leoditto/chromashell/releases/download/v#{version}/chromashell-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER"
    end
    on_intel do
      url "https://github.com/leoditto/chromashell/releases/download/v#{version}/chromashell-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/leoditto/chromashell/releases/download/v#{version}/chromashell-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER"
    end
    on_intel do
      url "https://github.com/leoditto/chromashell/releases/download/v#{version}/chromashell-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER"
    end
  end

  def install
    bin.install "chromashell"
    bin.install "cs"
  end

  test do
    assert_match "chromashell", shell_output("#{bin}/chromashell --version")
  end
end
