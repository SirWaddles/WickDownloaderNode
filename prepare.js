const fs = require('fs');
const neon_build = require('neon-cli/lib/ops/neon_build');
const nodepre = require('node-pre-gyp');
const nodepregit = require('node-pre-gyp-github');

// https://stackoverflow.com/a/32197381/3479580
function deleteFolderRecursive(path) {
    if (fs.existsSync(path)) {
        fs.readdirSync(path).forEach(function(file, index) {
            var curPath = path + "/" + file;
            if (fs.lstatSync(curPath).isDirectory()) { // recurse
                deleteFolderRecursive(curPath);
            } else { // delete file
                fs.unlinkSync(curPath);
            }
        });
        fs.rmdirSync(path);
    }
};

function PackageProgram() {
    return new Promise((resolve, reject) => {
        let builder = new nodepre.Run();
        builder.commands.package(process.argv, () => {
            resolve();
        });
    });
}

function PublishProgram() {
    if (!process.env['NODE_PRE_GYP_GITHUB_TOKEN']) {
        process.env['NODE_PRE_GYP_GITHUB_TOKEN'] = fs.readFileSync("./deploy_token.txt");
    }
    const publisher = new nodepregit();
    publisher.publish();
}

async function BuildProgram() {
    deleteFolderRecursive('./dist');
    fs.mkdirSync('./dist');
    await neon_build.default(process.cwd());
    fs.copyFileSync('./native/index.node', './dist/index.node');
}

async function RunPublish() {
    await PackageProgram();
    PublishProgram();
}

if (process.argv[2] == "build") {
    BuildProgram();
}

if (process.argv[2] == "publish") {
    RunPublish();
}
