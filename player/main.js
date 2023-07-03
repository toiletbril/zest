const endpoint = `http://localhost:6969/api/v1/music`;

let updatingBuffers = 0;
let totalDuration = 0;

const audioPlayer = document.getElementById("audio-player");

function fetchMusic(trackName) {
    let mediaSource = new MediaSource();
    audioPlayer.src = URL.createObjectURL(mediaSource);

    document.getElementById("track-name").textContent = trackName;

    mediaSource.addEventListener("sourceopen", () => {
        const url = endpoint + `/get?name=${encodeURIComponent(trackName)}&chunk=0`;
        let sourceBuffer = mediaSource.addSourceBuffer("audio/mpeg");
        appendChunks(sourceBuffer, mediaSource, url);
    });

    mediaSource.addEventListener("error", error => {
        console.error("Error opening media source:", error);
    });
}

function appendChunks(sourceBuffer, mediaSource, url) {
    fetch(url)
        .then(response => {
            if (!response.ok) {
                throw new Error("Error retrieving music chunk");
            }
            return response.arrayBuffer();
        })
        .then(arrayBuffer => {
            const appendInterval = setInterval(() => {
                if (!sourceBuffer.updating) {
                    clearInterval(appendInterval);

                    mediaSource.duration = totalDuration;
                    audioPlayer.duration = mediaSource.duration;

                    appendNextChunk(sourceBuffer, mediaSource, url, arrayBuffer);
                }
            }, 100);
        })
        .catch(error => console.error("Error fetching music chunk:", error));
}

const CHUNK_SIZE = 1024 * 128; // 128 kb

function appendNextChunk(sourceBuffer, mediaSource, url, arrayBuffer) {
    sourceBuffer.appendBuffer(arrayBuffer);

    const isLastChunk = arrayBuffer.byteLength < CHUNK_SIZE;

    if (!isLastChunk) {
        const nextChunkIndex = parseInt(new URL(url).searchParams.get("chunk")) + 1;
        const nextUrl = url.replace(/chunk=\d+/, `chunk=${nextChunkIndex}`);
        appendChunks(sourceBuffer, mediaSource, nextUrl);
    }

    extendDuration(arrayBuffer);
}

function extendDuration(arrayBuffer) {
    const audioContext = new AudioContext();
    audioContext.decodeAudioData(arrayBuffer)
        .then(decodedData => {
            totalDuration += decodedData.duration;
        });
}

function resetPlayback() {
    totalDuration = 0;
    updatingBuffers = 0;
}

function playTrack(trackName) {
    resetPlayback();
    fetchMusic(trackName);
    audioPlayer.currentTime = 0;
    audioPlayer.play();
}

let allTracks = [];

function fetchTrackNames() {
    fetch(endpoint + "/all")
        .then(response => response.json())
        .then(trackNames => {
            allTracks = trackNames;
            updateMusicList(allTracks);
        })
        .catch(error => console.error("Error fetching track names:", error));
}

const searchInput = document.getElementById("search-input");

function searchTracks(event) {
    const searchTerm = searchInput.value.trim().toLowerCase();
    if (searchTerm === "") {
        updateMusicList(allTracks);
    } else if (event.key === "Enter") {
        const filteredTracks = allTracks.filter(track => track.toLowerCase().includes(searchTerm));
        updateMusicList(filteredTracks);
    }
}

function updateMusicList(tracks) {
    if (tracks.length > 100) {
        tracks = tracks.slice(0, 100);
    }

    const tracksDiv = document.getElementById("tracks");
    tracksDiv.innerHTML = "";

    tracks.forEach(trackName => {
        const trackLink = document.createElement("a");
        trackLink.href = "#";
        trackLink.className = "track";
        trackLink.innerText = trackName;
        trackLink.addEventListener("click", () => {
            event.preventDefault();
            playTrack(trackName);
        });

        const trackItem = document.createElement("div");
        trackItem.appendChild(trackLink);
        tracksDiv.appendChild(trackItem);
    });
}

window.onload = () => {
    fetchTrackNames();

    searchInput.addEventListener("input", searchTracks);
    searchInput.addEventListener("keyup", searchTracks);

    if (!MediaSource.isTypeSupported("audio/mpeg")) {
        alert(
            "Your browser does not support decoding of audio/mpeg, and playing music will not work.\n\n" +
            "To fix this, please install a compatible decoder, such as ffmpeg.");
    }
};
