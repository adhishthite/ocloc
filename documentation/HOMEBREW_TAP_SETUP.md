# Homebrew Tap Setup Guide for ocloc

This guide explains how to set up a Homebrew tap to distribute `ocloc` via Homebrew.

## Prerequisites

- GitHub account with admin access to the `ocloc` repository
- Homebrew installed on your Mac (for testing)

## Step 1: Create the Tap Repository

1. Go to GitHub and create a new repository named `homebrew-ocloc`

   - URL will be: `https://github.com/adhishthite/homebrew-ocloc`
   - Make it public (required for public taps)
   - Initialize with a README

2. Clone the repository locally:

   ```bash
   git clone https://github.com/adhishthite/homebrew-ocloc.git
   cd homebrew-ocloc
   ```

3. Create the Formula directory structure:

   ```bash
   mkdir -p Formula
   touch Formula/.gitkeep
   git add Formula/.gitkeep
   git commit -m "Initial tap structure"
   git push origin main
   ```

## Step 2: Create a GitHub Personal Access Token

1. Go to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)

   - Or visit: <https://github.com/settings/tokens>

2. Click "Generate new token (classic)"

3. Configure the token:

   - **Note**: `ocloc homebrew tap automation`
   - **Expiration**: Choose based on your security preferences (90 days recommended)
   - **Scopes**: Select the following:
     - ✅ `repo` (Full control of private repositories)
     - ✅ `workflow` (Update GitHub Action workflows)

4. Click "Generate token" and **copy the token immediately** (you won't see it again)

## Step 3: Add Secrets to the ocloc Repository

1. Go to your `ocloc` repository on GitHub
2. Navigate to Settings → Secrets and variables → Actions
3. Click "New repository secret"

4. Add the following secrets:

   **TAP_TOKEN**

   - Name: `TAP_TOKEN`
   - Value: The personal access token you created in Step 2

5. Add repository variable (optional, if not using default):
   - Go to Settings → Secrets and variables → Actions → Variables tab
   - Click "New repository variable"
   - Name: `TAP_REPO`
   - Value: `adhishthite/homebrew-ocloc` (or your tap repo path)

## Step 4: Verify the Release Workflow

The `.github/workflows/release.yml` already includes the tap update job. It will:

- Download release artifacts
- Generate the Homebrew formula with correct URLs and checksums
- Push the formula to your tap repository

## Step 5: Test the Setup

1. Create a new release:

   ```bash
   # Update version in Cargo.toml
   git add Cargo.toml
   git commit -m "chore: bump version to X.Y.Z"
   git tag vX.Y.Z
   git push origin main
   git push origin vX.Y.Z
   ```

2. Monitor the GitHub Actions workflow:

   - Go to Actions tab in the `ocloc` repository
   - Watch the "Release" workflow
   - Verify all jobs complete successfully, including "Update Homebrew Tap"

3. Check the tap repository:
   - Visit `https://github.com/adhishthite/homebrew-ocloc`
   - Verify `Formula/ocloc.rb` was created/updated

## Step 6: Install via Homebrew

Once the tap is set up and a release is published:

```bash
# Add the tap (one-time setup for users)
brew tap adhishthite/ocloc

# Install ocloc
brew install ocloc

# Or as a one-liner
brew install adhishthite/ocloc/ocloc

# Upgrade to latest version
brew upgrade ocloc

# Uninstall
brew uninstall ocloc
```

## Troubleshooting

### Release workflow fails at "Update Homebrew Tap"

1. **Check secrets**: Ensure `TAP_TOKEN` is set correctly
2. **Token permissions**: Verify the token has `repo` scope
3. **Tap repository**: Ensure the tap repository exists and is accessible

### Formula not updating

1. **Check workflow logs**: Look for errors in the GitHub Actions logs
2. **Manual update**: You can manually update the formula:

   ```bash
   cd homebrew-ocloc
   # Edit Formula/ocloc.rb with correct URLs and checksums
   git add Formula/ocloc.rb
   git commit -m "ocloc X.Y.Z"
   git push origin main
   ```

### Installation fails for users

Common issues and solutions:

1. **"No available formula"**: Ensure the tap is added: `brew tap adhishthite/ocloc`
2. **Checksum mismatch**: The formula has incorrect SHA256 values - needs manual fix
3. **404 errors**: The release assets URLs are incorrect in the formula

## Formula Structure

The automated workflow generates a formula like this:

```ruby
class Ocloc < Formula
  desc "Fast, reliable lines-of-code counter"
  homepage "https://github.com/adhishthite/ocloc"
  version "X.Y.Z"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/adhishthite/ocloc/releases/download/vX.Y.Z/ocloc-X.Y.Z-aarch64-apple-darwin.tar.gz"
      sha256 "..."
    else
      url "https://github.com/adhishthite/ocloc/releases/download/vX.Y.Z/ocloc-X.Y.Z-x86_64-apple-darwin.tar.gz"
      sha256 "..."
    end
  end

  on_linux do
    url "https://github.com/adhishthite/ocloc/releases/download/vX.Y.Z/ocloc-X.Y.Z-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "..."
  end

  def install
    bin.install "ocloc"
  end

  test do
    system "#{bin}/ocloc", "--version"
  end
end
```

## Security Considerations

1. **Token rotation**: Rotate your PAT regularly (every 90 days recommended)
2. **Minimal scope**: Only grant the minimum required permissions
3. **Secret scanning**: GitHub will automatically revoke tokens if they're exposed in code
4. **Audit logs**: Regularly review your repository's audit logs for unauthorized access

## Advanced Configuration

### Custom tap name

If you want a different tap name (e.g., `homebrew-tools` for multiple tools):

1. Update the repository variable `TAP_REPO` to point to the new repository
2. Users would install via: `brew tap adhishthite/tools` and `brew install ocloc`

### Private taps

For private distribution:

1. Make the tap repository private
2. Users need GitHub authentication to access:

   ```bash
   export HOMEBREW_GITHUB_API_TOKEN=your_token
   brew tap adhishthite/private-tap
   ```

## Maintenance

- **Monitor releases**: Check that each release properly updates the tap
- **Test installations**: Periodically test fresh installations
- **Update documentation**: Keep installation instructions current in README
- **Clean old versions**: Homebrew formulas typically only maintain the latest version

## Additional Resources

- [Homebrew Tap Documentation](https://docs.brew.sh/Taps)
- [GitHub Actions Secrets](https://docs.github.com/en/actions/security-guides/encrypted-secrets)
- [Personal Access Tokens](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token)
