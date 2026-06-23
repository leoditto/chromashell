class Chromashell < Formula
  desc "Live shader effects for your terminal"
  homepage "https://github.com/leoditto/chromashell"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/leoditto/chromashell/releases/download/v#{version}/chromashell-aarch64-apple-darwin.tar.gz"
      sha256 "ebb59f5709c62ecef74b250c6640575bdb1382b18d2c7ef04774b3d958d4a55b"
    end
    on_intel do
      url "https://github.com/leoditto/chromashell/releases/download/v#{version}/chromashell-x86_64-apple-darwin.tar.gz"
      sha256 "a3e34fe322a3f9f67e06013829fe686c0081446a1329b454c57294c973c99daa"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/leoditto/chromashell/releases/download/v#{version}/chromashell-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "aa6fa8f17e7e5088f39d624b39a559562f7b32de1ba5ab57b98f9a2e147298e2"
    end
    on_intel do
      url "https://github.com/leoditto/chromashell/releases/download/v#{version}/chromashell-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "2a8c778e87f8e739b32a77506db23a730af46776cf17a0196209bac9743aeb6d"
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
