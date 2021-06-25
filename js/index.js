import("../pkg/index.js").catch(console.error);

const CLIENT_ID = '748463982862-m2rm6gga50bmmsqim4teptgo366k27b6.apps.googleusercontent.com';
const DISCOVERY_DOCS = ["https://www.googleapis.com/discovery/v1/apis/drive/v3/rest"];
const SCOPES = 'https://www.googleapis.com/auth/drive.appdata';
const filename = 'config.json'

let gdriveButton = document.getElementById('gdrive-button');

window.handleClientLoad = function () {
    gapi.load('client:auth2', function () {
        gapi.client
            .init({
                clientId: CLIENT_ID,
                discoveryDocs: DISCOVERY_DOCS,
                scope: SCOPES
            })
            .then(function () {
                gapi.auth2.getAuthInstance().isSignedIn.listen(updateSigninStatus);

                updateSigninStatus(gapi.auth2.getAuthInstance().isSignedIn.get());
                gdriveButton.onclick = function () {
                    if (gapi.auth2.getAuthInstance().isSignedIn.get()) {
                        gapi.auth2.getAuthInstance().signOut();
                    } else {
                        gapi.auth2.getAuthInstance().signIn();
                    }
                };

            }, function(error) {
                console.log(JSON.stringify(error, null, 2));
            });
    });
}

function getFileId(){
    return gapi.client.drive.files.list({
        q: 'name="' + filename + '"',
        spaces: 'appDataFolder',
        fields: "nextPageToken, files(id, name)"
    }).then(function(response) {
        let files = response.result.files;
        if(files && files.length > 0){
            return files[0].id;
        }
        return null;
    });
}

function createFileId() {
    return gapi.client.drive.files
        .create({
            fields: 'id',
            resource: { name: filename, parents: ['appDataFolder'] }
        })
        .then(function (response) {
            let file = response.result;
            return file ? file.id : null;
        });
}

function onSave(e) {
    console.log(e.detail);
    if (window.saveFileId !== undefined && window.saveFileId !== null){
        gapi.client.request({
            path: '/upload/drive/v3/files/' + window.saveFileId,
            method: 'PATCH',
            params: { uploadType: 'media' },
            body: e.detail
        }).then(function (res){
            console.log(res);
        });
    }
}

function updateSigninStatus(isSignedIn) {
    if (isSignedIn) {
        getFileId().then(function(id){
            if(id == null) {
                createFileId().then(function (nid){
                    window.saveFileId = nid;
                });
            } else {
                gapi.client.drive.files
                    .get({ fileId: id, alt: 'media' })
                    .then(function (response) {
                        window.saveFileId = id;
                        window.dispatchEvent(new CustomEvent('save-received', {detail: response.body.trim()}));
                    });
            }
        });
        window.addEventListener('saved', onSave, false);
        gdriveButton.innerText = 'Log Out'
    } else {
        window.removeEventListener('saved', onSave, false);
        gdriveButton.innerText = 'Log In'
    }
}


