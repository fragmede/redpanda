class Rp < Formula
  desc "Display images and text in terminal using kitty graphics protocol"
  homepage "https://github.com/fragmede/redpanda"
  url "https://github.com/fragmede/redpanda/archive/refs/heads/main.tar.gz"
  version "1.0.0"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "build", "--release"
    bin.install "target/release/redpanda" => "rp"
  end

  def caveats
    <<~EOS
      To use rp as your default cat, add to your shell config:
        alias cat=#{bin}/rp
    EOS
  end

  test do
    (testpath/"test.txt").write("hello world")
    assert_equal "hello world", shell_output("#{bin}/rp test.txt")
  end
end
