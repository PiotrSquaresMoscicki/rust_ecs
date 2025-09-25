# GitHub Actions Workflows

This directory contains the GitHub Actions workflows for the Rust ECS project.

## Workflows

### `ci.yml` - Continuous Integration
- Runs on pushes and PRs to `main`/`master` branches
- Tests the code with multiple Rust versions (stable, beta, nightly)
- Checks formatting and linting
- Validates MSRV (Minimum Supported Rust Version)

### `deploy.yml` - WebAssembly Deployment
- **Triggers**: 
  - Pushes to `dev` branch → Deploy to GitHub Pages
  - Pull requests to `dev` branch → Create preview deployments
- **Jobs**:
  - `build-wasm`: Builds WebAssembly module and uploads artifacts
  - `deploy-pages`: Deploys to GitHub Pages (push to dev only)
  - `deploy-pr-preview`: Creates Netlify preview deployment (PRs only)
  - `comment-pr`: Posts/updates PR comments with deployment status and links
- **Features**:
  - Builds WebAssembly module using `wasm-pack`
  - Deploys main site to GitHub Pages automatically
  - Creates preview deployments for PRs (with Netlify integration)
  - Posts comprehensive PR comments with deployment status and preview links
  - Updates existing comments instead of creating duplicates
  - Provides fallback instructions when Netlify is not configured
  - Handles both successful and failed deployments gracefully
  - **Enhanced caching** for faster builds (includes wasm-pack caching)
  - **Consistent action versions** for better compatibility
  - **Improved URL validation** to avoid empty/undefined preview links

## Setting Up Deployments

### GitHub Pages (Automatic)
GitHub Pages deployment works automatically once:
1. The workflow runs on a push to `dev` branch
2. Repository has Pages enabled in Settings → Pages → Source: "GitHub Actions"

### PR Preview Deployments (Optional)
PR previews use Netlify for hosting. To enable:

1. **Create a Netlify account** at https://netlify.com
2. **Create a new site** (can be empty, just for hosting)
3. **Get your credentials**:
   - Site ID: Found in Site Settings → General → Site information
   - Auth Token: User Settings → Applications → Personal access tokens → New access token
4. **Add secrets** in GitHub repo Settings → Secrets and variables → Actions:
   - `NETLIFY_AUTH_TOKEN`: Your personal access token
   - `NETLIFY_SITE_ID`: Your site ID

### Manual Testing (Always Available)
Even without Netlify setup, you can:
1. Download artifacts from workflow runs
2. Extract the files
3. Run `python3 -m http.server 8000` in the extracted directory
4. Visit `http://localhost:8000`

## Workflow Features

### Caching
Both workflows use comprehensive caching:
- Cargo registry and index
- Built dependencies
- Rust toolchains

### Security
- Uses official GitHub Actions
- Minimal required permissions
- Secrets are properly scoped
- Continue-on-error for optional features

### Error Handling
- Graceful fallbacks when external services fail
- Clear error messages in PR comments
- Artifact uploads always work regardless of deployment status

## Customization

### Changing Target Branch
To deploy from a different branch, update the `branches` list in `deploy.yml`:

```yaml
on:
  push:
    branches: [ your-branch ]  # Change this
  pull_request:
    branches: [ your-branch ]  # And this
```

### Build Configuration
The WebAssembly build uses:
- `--target web`: ES6 modules for direct browser use
- `--release`: Optimized builds for production
- Output to `www/pkg/`: Keeps web files together

You can modify the build command in the "Build WebAssembly" step.

### Preview URL Format
PR preview URLs follow the pattern: `rust-ecs-pr-{PR_NUMBER}.netlify.app`

This can be customized by changing the `alias` in the Netlify deployment step.

## Troubleshooting

### Common Issues

1. **Pages deployment fails**: 
   - Check that Pages is enabled in repository settings
   - Verify the workflow has `pages: write` permission

2. **Netlify preview fails**:
   - Check that secrets are correctly set
   - Verify the Site ID matches your Netlify site

3. **Build fails**:
   - Check Rust toolchain compatibility
   - Verify `wasm-pack` installation succeeds
   - Review WebAssembly dependencies

4. **Cache issues**:
   - Clear workflow caches in Actions tab → Caches
   - Update cache keys if dependencies change significantly

### Viewing Logs
- Go to Actions tab in GitHub repository
- Click on a workflow run to see detailed logs
- Each job shows individual step outputs
- Artifacts are available for download from successful runs

## Security Notes

- Secrets are only available to authorized users
- PR deployments from forks cannot access secrets (by design)
- All dependencies are installed fresh in each run
- No sensitive data is exposed in build artifacts