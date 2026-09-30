# Homebrew formula template for sgl.
# Replace VERSION, SHA256_MACOS_X86, SHA256_MACOS_ARM, SHA256_LINUX_X86, SHA256_LINUX_ARM
# with the values from the release's SHA256SUMS file.

class SingularityCli < Formula
  desc "One set of commands for every project type"
  homepage "https://getsingularity.lol/cli"
  version "VERSION"
  license "MIT"

  on_macos do
    on_intel do
      url "https://github.com/h4nxxlmao/singularity-cli/releases/download/vVERSION/sgl-vVERSION-x86_64-apple-darwin.tar.gz"
      sha256 "SHA256_MACOS_X86"
    end
    on_arm do
      url "https://github.com/h4nxxlmao/singularity-cli/releases/download/vVERSION/sgl-vVERSION-aarch64-apple-darwin.tar.gz"
      sha256 "SHA256_MACOS_ARM"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/h4nxxlmao/singularity-cli/releases/download/vVERSION/sgl-vVERSION-x86_64-unknown-linux-musl.tar.gz"
      sha256 "SHA256_LINUX_X86"
    end
    on_arm do
      url "https://github.com/h4nxxlmao/singularity-cli/releases/download/vVERSION/sgl-vVERSION-aarch64-unknown-linux-musl.tar.gz"
      sha256 "SHA256_LINUX_ARM"
    end
  end

  def install
    bin.install "sgl"
  end

  test do
    system "#{bin}/sgl", "--version"
  end
end
