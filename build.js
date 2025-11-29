#!/usr/bin/env node

/**
 * Cross-platform build script for rusty-g6
 * Simple automated build process from frontend to final executable
 */

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const os = require('os');

// Colors for console output
const colors = {
    reset: '\x1b[0m',
    bright: '\x1b[1m',
    red: '\x1b[31m',
    green: '\x1b[32m',
    yellow: '\x1b[33m',
    blue: '\x1b[34m',
    magenta: '\x1b[35m',
    cyan: '\x1b[36m'
};

function log(message, color = colors.reset) {
    console.log(`${color}${message}${colors.reset}`);
}

function logStep(step, message) {
    log(`\n${colors.bright}[${step}]${colors.reset} ${colors.cyan}${message}${colors.reset}`);
}

function logSuccess(message) {
    log(`${colors.green}✅ ${message}${colors.reset}`);
}

function logError(message) {
    log(`${colors.red}❌ ${message}${colors.reset}`);
}

function runCommand(command, cwd = process.cwd(), description = '') {
    try {
        if (description) {
            log(`   ${colors.yellow}${description}${colors.reset}`);
        }
        log(`   ${colors.blue}$ ${command}${colors.reset}`);
        
        execSync(command, { 
            cwd, 
            stdio: 'inherit',
            encoding: 'utf8'
        });
    } catch (error) {
        logError(`Command failed: ${command}`);
        process.exit(1);
    }
}

function checkBasicRequirements() {
    logStep('1', 'Checking basic requirements...');
    
    // Check if we're in the right directory
    if (!fs.existsSync('rust') || !fs.existsSync('rust/package.json')) {
        logError('This script must be run from the project root directory');
        logError('Expected structure: ./rust/package.json');
        process.exit(1);
    }
    
    // Check Node.js
    try {
        const nodeVersion = execSync('node --version', { encoding: 'utf8' }).trim();
        logSuccess(`Node.js: ${nodeVersion}`);
    } catch (error) {
        logError('Node.js not found. Please install from https://nodejs.org/');
        process.exit(1);
    }
    
    // Check Rust
    try {
        const rustVersion = execSync('rustc --version', { encoding: 'utf8' }).trim();
        logSuccess(`Rust: ${rustVersion}`);
    } catch (error) {
        logError('Rust not found. Please install from https://rustup.rs/');
        process.exit(1);
    }
    
    // Check Tauri CLI
    try {
        const tauriVersion = execSync('cargo tauri --version', { encoding: 'utf8' }).trim();
        logSuccess(`Tauri CLI: ${tauriVersion}`);
    } catch (error) {
        log(`   ${colors.yellow}Tauri CLI not found. Installing...${colors.reset}`);
        runCommand('cargo install tauri-cli', process.cwd(), 'Installing Tauri CLI');
        logSuccess('Tauri CLI installed successfully');
    }
    
    // Check Linux system dependencies
    if (os.platform() === 'linux') {
        checkLinuxDependencies();
    }
}

function checkLinuxDependencies() {
    log(`   ${colors.yellow}Checking Linux system dependencies...${colors.reset}`);
    
    const requiredPackages = [
        'libwebkit2gtk-4.1-dev',
        'libgtk-3-dev',
        'libayatana-appindicator3-dev',
        'librsvg2-dev',
        'build-essential',
        'curl',
        'wget',
        'file',
        'libssl-dev',
        'libudev-dev'
    ];
    
    const missingPackages = [];
    
    for (const pkg of requiredPackages) {
        try {
            execSync(`dpkg-query -W -f='${pkg}' ${pkg}`, { 
                encoding: 'utf8', 
                stdio: 'pipe' 
            });
        } catch (error) {
            missingPackages.push(pkg);
        }
    }
    
    if (missingPackages.length > 0) {
        logError('Missing required Linux system dependencies:');
        missingPackages.forEach(pkg => log(`   - ${pkg}`));
        log('\n   To install missing dependencies, run:');
        log(`   ${colors.blue}sudo apt update && sudo apt install -y ${missingPackages.join(' ')}${colors.reset}\n`);
        process.exit(1);
    } else {
        logSuccess('All Linux system dependencies are installed');
    }
}

function cleanBuild() {
    logStep('2', 'Cleaning previous build...');
    
    const distDir = path.join('rust', 'dist');
    
    if (fs.existsSync(distDir)) {
        log(`   Removing ${distDir}`);
        fs.rmSync(distDir, { recursive: true, force: true });
    }
    
    // Clean Rust target if --clean flag is used
    const args = process.argv.slice(2);
    if (args.includes('--clean') || args.includes('-c')) {
        const targetDir = path.join('rust', 'src-tauri', 'target');
        if (fs.existsSync(targetDir)) {
            log(`   Removing ${targetDir} (--clean flag)`);
            fs.rmSync(targetDir, { recursive: true, force: true });
        }
    }
    
    logSuccess('Cleanup completed');
}

function installDependencies() {
    logStep('3', 'Installing dependencies...');
    
    const rustDir = path.join('rust');
    runCommand('npm install', rustDir, 'Installing npm dependencies');
    
    logSuccess('Dependencies installed');
}

function buildApp() {
    logStep('4', 'Building Tauri application...');
    
    const rustDir = path.join('rust');
    const args = process.argv.slice(2);
    const isDev = args.includes('--dev') || args.includes('-d');
    
    if (isDev) {
        runCommand('cargo tauri build --debug', rustDir, 'Building Tauri app (debug mode)');
    } else {
        runCommand('cargo tauri build', rustDir, 'Building Tauri app (release mode)');
    }
    
    logSuccess('Application build completed');
}

function showResults() {
    logStep('5', 'Build completed! 🎉');
    
    const args = process.argv.slice(2);
    const isDev = args.includes('--dev') || args.includes('-d');
    const buildType = isDev ? 'debug' : 'release';
    
    const platform = os.platform();
    const executableName = platform === 'win32' ? 'rusty-g6.exe' : 'rusty-g6';
    const executablePath = path.join('rust', 'src-tauri', 'target', buildType, executableName);
    
    if (fs.existsSync(executablePath)) {
        const stats = fs.statSync(executablePath);
        const sizeInMB = (stats.size / (1024 * 1024)).toFixed(2);
        
        log('\n📦 Build Results:');
        logSuccess(`Executable: ${executablePath}`);
        log(`   Size: ${sizeInMB} MB`);
        
        log('\n🚀 To run the application:');
        if (platform === 'win32') {
            log(`   .\\rust\\src-tauri\\target\\${buildType}\\rusty-g6.exe`);
        } else {
            log(`   ./rust/src-tauri/target/${buildType}/rusty-g6`);
        }
    } else {
        logError(`Expected executable not found at: ${executablePath}`);
    }
}

function showUsage() {
    log(`
${colors.bright}rusty-g6 Build Script${colors.reset}

${colors.cyan}Usage:${colors.reset}
  node build.js [options]

${colors.cyan}Options:${colors.reset}
  --dev, -d     Build in development mode (faster, larger binary)
  --clean, -c   Clean all build artifacts before building
  --help, -h    Show this help message

${colors.cyan}Examples:${colors.reset}
  node build.js                 # Standard release build
  node build.js --dev           # Development build
  node build.js --clean         # Clean build
  node build.js --dev --clean   # Clean development build

${colors.cyan}What this script does:${colors.reset}
  1. Checks basic requirements (Node.js, Rust, Tauri CLI)
  2. Cleans previous build artifacts
  3. Installs dependencies (npm install)
  4. Builds Tauri application (cargo tauri build)
  5. Shows build results and executable location
`);
}

function main() {
    const args = process.argv.slice(2);
    
    if (args.includes('--help') || args.includes('-h')) {
        showUsage();
        return;
    }
    
    log(`${colors.bright}${colors.magenta}🔨 rusty-g6 Build Script${colors.reset}`);
    log(`${colors.cyan}Simple cross-platform build automation${colors.reset}\n`);
    
    const startTime = Date.now();
    
    try {
        checkBasicRequirements();
        cleanBuild();
        installDependencies();
        buildApp();
        showResults();
        
        const endTime = Date.now();
        const duration = ((endTime - startTime) / 1000).toFixed(1);
        
        log(`\n${colors.green}${colors.bright}✅ Build completed successfully in ${duration}s!${colors.reset}`);
        
    } catch (error) {
        logError(`Build failed: ${error.message}`);
        process.exit(1);
    }
}

// Run the script
if (require.main === module) {
    main();
}
