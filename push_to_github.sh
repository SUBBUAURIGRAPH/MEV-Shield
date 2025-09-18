#!/bin/bash

# MEV Shield - Final Push to GitHub
# Repository: SUBBUAURIGRAPH/MEV-Shield

echo "════════════════════════════════════════════════════════════════"
echo "         🛡️  MEV Shield - Pushing to GitHub                     "
echo "════════════════════════════════════════════════════════════════"
echo ""

# Navigate to project directory
cd '/Users/subbujois/subbuworkingdir/MEV Shield/MEV-Shield' || exit 1

echo "📍 Current directory: $(pwd)"
echo ""

# Step 1: Set remote URL
echo "1️⃣ Setting remote URL to SSH..."
git remote set-url origin git@github.com:SUBBUAURIGRAPH/MEV-Shield.git
echo "✅ Remote URL set to: git@github.com:SUBBUAURIGRAPH/MEV-Shield.git"
echo ""

# Step 2: Rename branch to main
echo "2️⃣ Ensuring branch is named 'main'..."
git branch -M main
echo "✅ Branch renamed to 'main'"
echo ""

# Step 3: Push to origin
echo "3️⃣ Pushing to GitHub..."
echo "─────────────────────────────────────────"
git push -u origin main

if [ $? -eq 0 ]; then
    echo ""
    echo "════════════════════════════════════════════════════════════════"
    echo "        🎉 SUCCESS! MEV Shield pushed to GitHub!                "
    echo "════════════════════════════════════════════════════════════════"
    echo ""
    echo "📊 Repository Information:"
    echo "─────────────────────────────────────────"
    echo "📁 Repository: SUBBUAURIGRAPH/MEV-Shield"
    echo "🌐 URL: https://github.com/SUBBUAURIGRAPH/MEV-Shield"
    echo "🔧 Branch: main"
    echo ""
    echo "📋 Next Steps:"
    echo "─────────────────────────────────────────"
    echo "1. View repository: https://github.com/SUBBUAURIGRAPH/MEV-Shield"
    echo "2. Create release: git tag -a v1.0.0 -m 'Release v1.0.0'"
    echo "3. Push tag: git push origin v1.0.0"
    echo "4. Enable GitHub Actions in repository settings"
    echo "5. Add README badges and update description"
    echo ""
    echo "🏷️ To create and push version tag:"
    echo "git tag -a v1.0.0 -m 'Release v1.0.0 - MEV Shield with Neural Enhancement'"
    echo "git push origin v1.0.0"
    echo ""
    echo "✨ MEV Shield v1.0.0 is now live on GitHub! ✨"
else
    echo ""
    echo "════════════════════════════════════════════════════════════════"
    echo "        ⚠️  Push encountered an issue                            "
    echo "════════════════════════════════════════════════════════════════"
    echo ""
    echo "🔧 Troubleshooting Steps:"
    echo "─────────────────────────────────────────"
    echo ""
    echo "1. Check SSH connection:"
    echo "   ssh -T git@github.com"
    echo ""
    echo "2. If 'Repository not found', create it first:"
    echo "   • Go to: https://github.com/new"
    echo "   • Repository name: MEV-Shield"
    echo "   • DON'T initialize with README"
    echo ""
    echo "3. If 'Permission denied (publickey)', set up SSH key:"
    echo "   ssh-keygen -t ed25519 -C 'your-email@example.com'"
    echo "   cat ~/.ssh/id_ed25519.pub"
    echo "   • Copy output and add to: https://github.com/settings/keys"
    echo ""
    echo "4. If 'failed to push some refs', pull first:"
    echo "   git pull origin main --allow-unrelated-histories"
    echo "   git push -u origin main"
    echo ""
    echo "5. Force push (use carefully!):"
    echo "   git push -u origin main --force"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
