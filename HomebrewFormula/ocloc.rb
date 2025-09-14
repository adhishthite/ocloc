class Ocloc < Formula
  desc "Fast, reliable lines-of-code counter"
  homepage "https://github.com/adhishthite/ocloc"
  version "0.1.0"
  if OS.mac?
    url "https://github.com/adhishthite/ocloc/releases/download/v0.1.0/ocloc-macos"
    sha256 "PUT_SHA256_OF_MACOS_BINARY_HERE"
  elsif OS.linux?
    url "https://github.com/adhishthite/ocloc/releases/download/v0.1.0/ocloc-linux"
    sha256 "PUT_SHA256_OF_LINUX_BINARY_HERE"
  end
  license "MIT"

  def install
    if OS.mac?
      bin.install "ocloc-macos" => "ocloc"
    else
      bin.install "ocloc-linux" => "ocloc"
    end
  end

  test do
    system "#{bin}/ocloc", "--version"
  end
end

