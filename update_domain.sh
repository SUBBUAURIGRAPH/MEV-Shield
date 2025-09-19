#!/bin/bash

# Update all .io references to .ai
echo "🔄 Updating all mevshield.io references to mevshield.ai..."

# Navigate to project directory
cd '/Users/subbujois/subbuworkingdir/MEV Shield/MEV-Shield' || exit 1

# Find and replace in all markdown files
find . -name "*.md" -type f -exec sed -i '' 's/mevshield\.io/mevshield.ai/g' {} \;
find . -name "*.md" -type f -exec sed -i '' 's/dev@mevshield\.io/dev@mevshield.ai/g' {} \;
find . -name "*.md" -type f -exec sed -i '' 's/demo\.mevshield\.io/demo.mevshield.ai/g' {} \;
find . -name "*.md" -type f -exec sed -i '' 's/docs\.mevshield\.io/docs.mevshield.ai/g' {} \;
find . -name "*.md" -type f -exec sed -i '' 's/api\.mevshield\.io/api.mevshield.ai/g' {} \;
find . -name "*.md" -type f -exec sed -i '' 's/dashboard\.mevshield\.io/dashboard.mevshield.ai/g' {} \;

echo "✅ All references updated to mevshield.ai"

# Show what was changed
echo ""
echo "📊 Files updated:"
grep -r "mevshield.ai" . --include="*.md" | head -10

echo ""
echo "✅ Domain update complete!"
echo ""
echo "Next steps:"
echo "1. Update DNS records for mevshield.ai"
echo "2. Set up subdomains (demo, docs, api, dashboard)"
echo "3. Configure SSL certificates"
echo "4. Update social media profiles"
