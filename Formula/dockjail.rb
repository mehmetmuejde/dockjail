# Formula template. scripts/render-formula.sh fills in the placeholders; the release workflow writes
# the result to Formula/dockjail.rb on main, so this repository is its own Homebrew tap.
class Dockjail < Formula
  desc "Keeps the macOS Dock locked to one display"
  homepage "https://github.com/mehmetmuejde/dockjail"
  url "https://github.com/mehmetmuejde/dockjail/releases/download/v0.1.3/dockjail-0.1.3-universal-apple-darwin.tar.gz"
  sha256 "8d8651c0778b34339ed010b913c5595b52a0a5ac504f79eb84c40e0bc4eb3686"
  license "MIT"

  depends_on :macos

  def install
    bin.install "dockjail"
  end

  def caveats
    <<~EOS
      Lock the Dock to this window's display: dockjail on
      List displays and choose one:           dockjail displays / dockjail on INDEX
      Unlock:                                 dockjail off

      dockjail needs the Accessibility permission (System Settings → Privacy &
      Security → Accessibility), twice:
        1. For your terminal app (Terminal, iTerm, Warp, ...): run `dockjail check`,
           enable the app, then quit and reopen it.
        2. For the login agent: `dockjail on` starts it, and it adds `dockjail`
           to the list. Enable that entry.
      After an upgrade macOS asks for it again.
    EOS
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/dockjail --version")
  end
end
