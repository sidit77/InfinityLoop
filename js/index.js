import("../pkg/index.js").catch(console.error);

const CLIENT_ID = '748463982862-m2rm6gga50bmmsqim4teptgo366k27b6.apps.googleusercontent.com';

const DISCOVERY_DOCS = ["https://www.googleapis.com/discovery/v1/apis/drive/v3/rest"];

const SCOPES = 'https://www.googleapis.com/auth/drive.appdata';

const filename = 'config.json'

let gdriveButton = document.getElementById('gdrive-button');

gapi.load('client:auth2', function () {
    gapi.client.init({
        clientId: CLIENT_ID,
        discoveryDocs: DISCOVERY_DOCS,
        scope: SCOPES
    }).then(function () {
        // Listen for sign-in state changes.
        gapi.auth2.getAuthInstance().isSignedIn.listen(updateSigninStatus);

        // Handle the initial sign-in state.
        updateSigninStatus(gapi.auth2.getAuthInstance().isSignedIn.get());
        gdriveButton.onclick = handleLogInOutClick;

        if(gapi.auth2.getAuthInstance().isSignedIn.get()){
            gapi.client.drive.files.list({
                q: 'name="' + filename + '"',
                spaces: 'appDataFolder',
                fields: "nextPageToken, files(id, name)"
            }).then(function(response) {
                let files = response.result.files;
                if(files && files.length > 0){
                    let id = files[0].id;
                    gapi.client.drive.files
                        .get({ fileId: id, alt: 'media' })
                        .then(function (response) {
                            console.log(response.body);
                            console.log(JSON.parse(response.body.trim()).counter);
                        });
                }
            });
        }

    }, function(error) {
        console.log(JSON.stringify(error, null, 2));
    });
});

function updateSigninStatus(isSignedIn) {
    if (isSignedIn) {
        gdriveButton.innerText = 'Log Out'
    } else {
        gdriveButton.innerText = 'Log In'
    }
}

function handleLogInOutClick(event) {
    if (gapi.auth2.getAuthInstance().isSignedIn.get()) {
        gapi.auth2.getAuthInstance().signOut();
    } else {
        gapi.auth2.getAuthInstance().signIn();
    }
}
