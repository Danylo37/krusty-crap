/*
   USER VIEW
   -> WHATSAPP: SEND MESSAGE, CREATING AND SWITCHING BETWEEN CHATS
*/

interface ChatHistory {
    Speaker: string;
    Message: string;
}

interface Phonebooks {
    [serverId: string]: string[];
}

interface FileList {
    [fileName: string]: string;
}

interface ServerData {
    name: string;
    files: FileList;
}

interface MediaItem {
    reference: string;
    media?: string;
}

interface MediaRef {
    [reference: string]: string;
}

// Access globals via window as any to avoid module augmentation issues
const win = window as any;

function openChat(chatName: string, clientId: string): void {
    // Update the chat header with the selected chat name.
    const chatHeader = document.getElementById('chat-header');
    if (chatHeader) chatHeader.textContent = chatName;

    // Remove the update dot from the clicked chat item and mark it as active.
    const chatItem = document.querySelector(`.chat-item[data-id="${clientId}"]`) as HTMLElement;
    if (chatItem) {
        const chatMessages = document.getElementById('chat-messages') as HTMLElement;
        if (chatMessages && chatMessages.style.background !== 'url("content_objects/whatsapp_bg.jpg")') {
            chatMessages.style.background = 'url("content_objects/whatsapp_bg.jpg")';
            chatMessages.style.backgroundSize = 'cover';
            chatMessages.style.backgroundPosition = 'center';
        }

        if (chatMessages) chatMessages.innerHTML = '';
        // Hide the update dot.
        const updateDot = chatItem.querySelector('.update-dot') as HTMLElement;
        if (updateDot) {
            updateDot.style.display = 'none';
        }

        // Remove 'active' class from any other chat items.
        document.querySelectorAll('.chat-item').forEach(item => item.classList.remove('active'));
        // Mark this chat as active.
        chatItem.classList.add('active');

        // Check if there is stored chat history for this chat item.
        if (chatItem.dataset.history) {
            const history = JSON.parse(chatItem.dataset.history) as ChatHistory[];
            updateChatWindow(history);
        }
    }
}

// Phonebooks STORED
const phonebooks: Phonebooks = {};
// Expose to inline scripts in index.html
(window as any).phonebooks = phonebooks;

/*
const phonebooks = {
   "0": ["0", "2", "5"],
   "1": ["1", "3", "4"],
   "2": ["7", "10", "15"],
};
*/

function createNewChat(): void {
    const chatPopup = document.getElementById('chat-popup') as HTMLElement;
    const receiverList = document.getElementById('receiver-list') as HTMLElement;

    if (!chatPopup || !receiverList) return;

    const listBoxes: HTMLSelectElement[] = [];

    // Create list boxes dynamically for each server
    Object.keys(phonebooks).forEach((serverId) => {
        receiverList.innerHTML = '';

        // Server container
        const serverContainer = document.createElement('div');
        serverContainer.id = `server-container-${serverId}`;
        serverContainer.style.cssText = "margin-bottom: 15px;";

        //Container name + search
        const labelContainer = document.createElement('div');
        labelContainer.style.cssText = "display:flex; align-items: center; justify-content: space-between; margin-bottom: 5px;";

        // Server label
        const serverLabel = document.createElement('label');
        serverLabel.textContent = `Phonebook id: ${serverId}: `;
        serverLabel.style.fontWeight = "bold";
        labelContainer.appendChild(serverLabel);

        // Create an image element for search.png and insert it next to the label
        const searchImg = document.createElement('img');
        searchImg.src = "content_objects/reload.png"; // Adjust path if needed
        searchImg.alt = "Search";
        searchImg.className = "comm-ui-reload-button";
        searchImg.id = `reload-${serverId}`;
        searchImg.onclick = function(event: Event) {
            (this as HTMLImageElement).style.animation = "spin 1s linear infinite";
            askListRegisteredClientsToServer(win.currentClientId, serverId);
        };
        labelContainer.appendChild(searchImg);

        //adding both to flex div
        serverContainer.appendChild(labelContainer);

        // Server list box
        const listBox = document.createElement('select');
        listBox.className = 'server-list-box';
        listBox.id = `server-list-box-${serverId}`;
        listBox.style.cssText = "margin-left: 10px; padding: 5px;";

        // Default "Not Choosed" option
        const defaultOption = document.createElement('option');
        defaultOption.value = '';
        defaultOption.textContent = 'Not Choosed';
        listBox.appendChild(defaultOption);

        // Gather existing chats (assuming each chat item's text is the receiver id)
        const chatList = document.getElementById('chat-list');
        const existingChats = chatList ? Array.from(chatList.children)
            .map(item => (item as HTMLElement).dataset.id) : [];

        if (Array.isArray(phonebooks[serverId])) {
            phonebooks[serverId].forEach((receiver) => {
                if (existingChats.includes(receiver.toString())) {
                    // Skip if a chat with this receiver already exists
                    return;
                }
                const option = document.createElement('option');
                option.value = receiver;
                option.textContent = receiver;
                listBox.appendChild(option);
            });
        } else {
            console.error("Expected an array for phonebooks[" + serverId + "], but got", phonebooks[serverId]);
        }

        // Add change event to handle exclusivity among list boxes
        listBox.addEventListener('change', () => {
            listBoxes.forEach((box) => {
                if (box !== listBox) {
                    box.value = ''; // Reset others
                }
            });
        });

        listBoxes.push(listBox);

        // Append the list box to the server container
        serverContainer.appendChild(listBox);

        // Append the server container to the receiver list
        receiverList.appendChild(serverContainer);
    });

    chatPopup.style.display = 'flex'; // Show the popup
}

function confirmSelection(): void {
    const listBoxes = document.querySelectorAll('.server-list-box') as NodeListOf<HTMLSelectElement>;
    let selectedReceiver = '';

    // Get the selected receiver (if any) from the list boxes
    listBoxes.forEach((box) => {
        if (box.value) {
            selectedReceiver = box.value;
        }
    });

    if (!selectedReceiver) {
        alert('Please select a receiver.');
        return;
    }

    createChatItem(selectedReceiver);

    // Optionally, open the new chat immediately
    openChat(selectedReceiver, selectedReceiver);

    // Close the chat pop-up
    closeChatPopup();
}

function createChatItem(clientId: string): void {
    // Create a new chat item in the chat list (sidebar)
    const chatList = document.getElementById('chat-list');
    if (!chatList) return;

    const chatItem = document.createElement('div');
    chatItem.className = 'chat-item';
    chatItem.dataset.id = clientId; // assign the data-id attribute

    // Include both the chat name and the update dot
    chatItem.innerHTML = `<span class="chat-name">${clientId}</span>
                         <span class="update-dot" style="display: none;"></span>`;

    // Set the onclick handler to open the chat and remove the notification dot
    chatItem.onclick = () => {
        openChat(clientId, clientId);
        // Remove the update dot when the chat is selected:
        const dot = chatItem.querySelector('.update-dot') as HTMLElement;
        if (dot) dot.style.display = 'none';
    };

    chatList.appendChild(chatItem);
}

function closeChatPopup(): void {
    const chatPopup = document.getElementById('chat-popup') as HTMLElement;
    if (chatPopup) chatPopup.style.display = 'none'; // Hide the popup
}

/*
   Requesting and Updating
*/

// Requesting
function askListRegisteredClientsToServer(clientId: string, serverId: string): void {
    // CHen/Chen/CHEN function to retrieve List registered clients
    if (win.ws && win.ws.readyState === WebSocket.OPEN) {
        // Construct the message with the actual values of whichClient and whichServer
        const message = {
            WsAskListRegisteredClientsToServer: {
                client_id: clientId.toString(), // Ensure u64 is sent as a string
                server_id: serverId.toString(), // Ensure u64 is sent as a string
            }
        };
        console.log('Sending WsAskFileList', message);
        win.ws.send(JSON.stringify(message));
    } else {
        console.error('WebSocket is not open. Unable to send update command.');
    }
}

const messageInput = document.getElementById('message-input') as HTMLInputElement;
if (messageInput) {
    messageInput.addEventListener('keydown', (event: KeyboardEvent) => {
        if (event.key === 'Enter') {
            sendMessage();
        }
    });
}

function sendMessage(): void {
    const messageInput = document.getElementById('message-input') as HTMLInputElement;
    const chatMessages = document.getElementById('chat-messages') as HTMLElement;
    if (!messageInput || !chatMessages) return;

    const messageText = messageInput.value.trim();
    const activeChatItem = document.querySelector('.chat-item.active') as HTMLElement;

    if (!messageText) {
        alert("Insert something in input");
        return;
    } // Do nothing if message is empty

    if (activeChatItem && activeChatItem.dataset.id) {
        sendMessageController(win.currentClientId, activeChatItem.dataset.id, messageText);
    }
    
    // Create the sent message element and add it to the chat window.
    const messageDiv = document.createElement('div');
    messageDiv.className = 'message sent';
    //messageDiv.style = 'margin-bottom: 10px; background-color: #007bff; padding: 10px; border-radius: 10px; max-width: 60%; color: white; align-self: flex-end;';
    messageDiv.textContent = messageText;
    chatMessages.appendChild(messageDiv);

    // Scroll to the bottom of the chat window
    chatMessages.scrollTop = chatMessages.scrollHeight;

    // Clear the input field
    messageInput.value = '';

    // Update the history of the active chat.
    if (activeChatItem) {
        // Read the current history or start with an empty array.
        let history: ChatHistory[] = [];
        if (activeChatItem.dataset.history) {
            try {
                history = JSON.parse(activeChatItem.dataset.history) as ChatHistory[];
            } catch (e) {
                console.error("Error parsing chat history:", e);
                history = [];
            }
        }
        // Add the new message to the history.
        history.push({Speaker: "Me", Message: messageText});
        // Save the updated history back as a JSON string.
        activeChatItem.dataset.history = JSON.stringify(history);
    }
}

function sendMessageController(sourceClientId: string, destClientId: string, messageText: string): void {
    //Chen sending message controller
    if (win.ws && win.ws.readyState === WebSocket.OPEN) {
        // Construct the message with the actual values of whichClient and whichServer
        const message = {
            WsSendMessage: {
                source_client_id: sourceClientId.toString(), // Ensure u64 is sent as a string
                dest_client_id: destClientId.toString(),     // Ensure u64 is sent as a string
                message: messageText,
            }
        };
        console.log('Sending WsAskFileContent', message);
        win.ws.send(JSON.stringify(message));
    } else {
        console.error('WebSocket is not open. Unable to send update command.');
    }
}

// UPDATING

function updateChatReceivers(HashListReceivers: { [serverId: string]: string[] }): void {
    for (const [serverId, listReceivers] of Object.entries(HashListReceivers)) {
        // Update the phonebooks object.
        phonebooks[serverId] = listReceivers;

        // Stop the reload animation, if present.
        const reloadElem = document.getElementById(`reload-${serverId}`) as HTMLElement;
        if (reloadElem) {
            reloadElem.style.animation = "";
        }

        // Update the list box options.
        const listBox = document.getElementById(`server-list-box-${serverId}`) as HTMLSelectElement;
        if (listBox) {
            // Clear existing options.
            listBox.innerHTML = "";

            const defaultOption = document.createElement('option');
            defaultOption.value = '';
            defaultOption.textContent = 'Not Choosed';
            listBox.appendChild(defaultOption);

            // Gather existing chats (assuming each chat item's text is the receiver id)
            const chatList = document.getElementById('chat-list');
            const existingChats = chatList ? Array.from(chatList.children)
                .map(item => (item as HTMLElement).dataset.id) : [];

            if (Array.isArray(phonebooks[serverId])) {
                phonebooks[serverId].forEach((receiver) => {
                    if (existingChats.includes(receiver.toString())) {
                        // Skip if a chat with this receiver already exists
                        return;
                    }
                    const option = document.createElement('option');
                    option.value = receiver;
                    option.textContent = receiver;
                    listBox.appendChild(option);
                });
            } else {
                console.error("Expected an array for phonebooks[" + serverId + "], but got", phonebooks[serverId]);
            }
        }
    }
}

function updateChats(ChatHistory: { [clientId: string]: ChatHistory[] }): void {
    const chatList = document.getElementById('chat-list');
    if (!chatList) return;

    const old_histories = Array.from(chatList.children)
        .reduce((acc: { [key: string]: string }, item) => {
            const element = item as HTMLElement;
            if (element.dataset.id && element.dataset.history) {
                acc[element.dataset.id] = element.dataset.history;
            }
            return acc;
        }, {});

    // Find the chat items that changed.
    for (const clientId in ChatHistory) {
        const newHistory = ChatHistory[clientId];
        if (!old_histories[clientId]) {
            createChatItem(clientId);
            const chatItem = document.querySelector(`.chat-item[data-id="${clientId}"]`) as HTMLElement;
            if (chatItem) {
                const updateDot = chatItem.querySelector('.update-dot') as HTMLElement;
                if (updateDot) {
                    updateDot.style.display = 'inline-block';
                }
            }
        }
        if (JSON.stringify(newHistory) !== old_histories[clientId]) {
            const chatItem = document.querySelector(`.chat-item[data-id="${clientId}"]`) as HTMLElement;
            if (chatItem) {
                chatItem.dataset.history = JSON.stringify(newHistory);

                if (chatItem.classList.contains('active')) {
                    // If active, update the chat window immediately.
                    updateChatWindow(newHistory);
                } else {
                    // If not active, show the update dot.
                    const updateDot = chatItem.querySelector('.update-dot') as HTMLElement;
                    if (updateDot) {
                        updateDot.style.display = 'inline-block';
                    }
                }
            }
        }
    }
}

function updateChatWindow(history: ChatHistory[]): void {
    const chatMessages = document.getElementById('chat-messages') as HTMLElement;
    if (!chatMessages) return;

    chatMessages.innerHTML = ""; // Clear current messages

    history.forEach(msg => {
        let speaker: string, message: string;
        if (Array.isArray(msg)) {
            // If msg is an array, assume the first element is the speaker and the second is the message.
            speaker = msg[0];
            message = msg[1];
        } else {
            // Otherwise, assume it's an object.
            speaker = msg.Speaker;
            message = msg.Message;
        }
        const messageDiv = document.createElement('div');
        messageDiv.className = (speaker === "Me") ? "message sent" : "message received";
        messageDiv.textContent = message;
        chatMessages.appendChild(messageDiv);
    });

    // Scroll to the bottom of the chat messages container.
    chatMessages.scrollTop = chatMessages.scrollHeight;
}

/*
   USER VIEW
   -> CONTENT APP: General for now
*/

// SERVER DATA STRUCTURE (Each server has its own files)
const file_lists: { [serverId: string]: ServerData } = {
    /*
    {
        name: "Server1",
        files: {
            "Document1.pdf": "This is the content of Document1.pdf...",
            "Image.png": "This file is an image, preview not available.",
            "Presentation.pptx": "Presentation about our latest project...",
            "Spreadsheet.xlsx": "Spreadsheet data showing company profits..."
        }
    },
    {
        name: "Server2",
        files: {
            "Report.docx": "Annual financial report...",
            "Photo.jpg": "Vacation photo...",
            "Notes.txt": "Meeting notes..."
        }
    },
    {
        name: "Server3",
        files: {
            "Music.mp3": "Favorite song...",
            "Video.mp4": "A short movie...",
            "Slides.pptx": "Presentation for class..."
        }
    }*/
};
// Expose to inline scripts in index.html
(window as any).file_lists = file_lists;

const media: MediaItem[] = [
    /*
    {
        reference : ""
    },
    */
];

// TRACK CURRENT SERVER INDEX
let currentServerIndex = 0;

// FILTER CURRENT and SEARCH CURRENT
let currentFilterType = ""; // An empty string means no filter is applied.
let currentSearchValue = "";

// FUNCTION TO NAVIGATE SERVERS
function navigateServer(direction: number): void {
    const serverKeys = Object.keys(file_lists);

    if (direction > 0) {
        if ((currentServerIndex + 1) === serverKeys.length) {
            currentServerIndex = 0;
        } else {
            currentServerIndex = currentServerIndex + 1;
        }
    } else {
        if ((currentServerIndex - 1) < 0) {
            currentServerIndex = serverKeys.length - 1;
        } else {
            currentServerIndex = currentServerIndex - 1;
        }
    }
    updateServerDisplay();
}

function get_server_id_from_current_server_index(): string {
    const serverKeys = Object.keys(file_lists);
    return serverKeys[currentServerIndex];
}

// FUNCTION TO UPDATE UI WHEN SWITCHING SERVERS
function updateServerDisplay(): void {
    const currentServerId = get_server_id_from_current_server_index();
    const currentServer = file_lists[currentServerId];

    if (!currentServer) return;

    // Update Server Name
    const currentServerElement = document.getElementById("current-server");
    if (currentServerElement) currentServerElement.textContent = currentServer.name;

    // Update File List
    updateFileList(currentServer.files);
}

// Attach Click Event to Each File Row
document.addEventListener("DOMContentLoaded", function () {
    document.querySelectorAll(".file-list tr").forEach(row => {
        row.addEventListener("click", function (this: HTMLTableRowElement) {
            const fileName = this.cells[0]?.textContent?.trim(); // Get file name from row
            if (fileName) openPopup(fileName);
        });
    });
});

// FUNCTION TO OPEN POP-UP WITH FILE CONTENT
function openPopup(fileName: string): void {
    const popup = document.getElementById("file-popup") as HTMLElement;
    const popupTitle = document.getElementById("file-popup-title") as HTMLElement;
    const popupFileContent = document.getElementById("file-popup-file-content") as HTMLElement;

    if (!popup || !popupTitle || !popupFileContent) return;

    // Get the current server and file content.
    //console.log(file_lists)
    //console.log(get_server_id_from_current_server_index())
    //console.log(file_lists[get_server_id_from_current_server_index()])

    const currentServer = file_lists[get_server_id_from_current_server_index()];
    //console.log(currentServer.files)
    //console.log(fileName)
    const fileContent = currentServer?.files[fileName];

    popupTitle.textContent = fileName;
    popupFileContent.innerHTML = '';

    const fullPath = window.location.pathname;
    // Remove the filename (assumes a filename is present)
    const basePath = fullPath.substring(0, fullPath.lastIndexOf('/'));
    // Combine with the protocol
    const absolutePath = window.location.protocol + basePath;

    //console.log(fileContent)
    if (fileContent) {
        // Determine the file extension.
        const extension = fileName.split('.').pop()?.toLowerCase();
        if (extension === "html") {
            // Use a regular expression to find all occurrences of #Media[...] in the content.
            const processedContent = fileContent.replace(/#Media\[(.*?)\]/g, (match, p1) => {
                const found = media.find(item => item.reference === p1);
                console.log(media);
                console.log(found);
                // Use the already loaded media image.
                if (found && found.media) {
                    return `<img src="${absolutePath + found.media}" id="reference-${p1}" alt="Media loaded" class="image-loaded" />`;
                }
                return match;
            });
            // Insert the processed HTML into the popup.
            popupFileContent.innerHTML = processedContent;
        } else if (extension === "mp3") {
            // For audio files, embed an HTML5 audio element.
            popupFileContent.innerHTML = `
              <audio controls style="width:100%;">
                <source src="${fileContent}" type="audio/mpeg">
                Your browser does not support the audio element.
              </audio>
            `;
        } else if (extension === "jpg" || extension === "jpeg" || extension === "png" || extension === "gif") {
            // For images, use an img element.
            popupFileContent.innerHTML = `<img src="${fileContent}" alt="${fileName}" style="max-width:100%;" />`;
        } else {
            // For all other types, simply display the content as text.
            popupFileContent.textContent = fileContent;
        }
    } else {
        popupFileContent.innerHTML = '';
        askFileContent(win.currentClientId, get_server_id_from_current_server_index(), fileName);
    }

    popup.style.display = "flex";
}

// Close the Pop-Up
function closePopup(): void {
    const popup = document.getElementById("file-popup") as HTMLElement;
    if (popup) popup.style.display = "none";
}

// Reload
function reloadFilesServer(whichServer: string): void {
    // Call the empty function
    askFileList(win.currentClientId, whichServer);

    // Show the loading overlay pop-up
    const loadingPopup = document.getElementById("loading-popup") as HTMLElement;
    if (loadingPopup) loadingPopup.style.display = "flex";
}

//ASKING

function askFileList(clientId: string, serverId: string): void {
    if (win.ws && win.ws.readyState === WebSocket.OPEN) {
        // Use the correct format for serde deserialization
        const message = {
            WsAskFileList: {
                client_id: clientId.toString(), // Ensure u64 is sent as a string
                server_id: serverId.toString(), // Ensure u64 is sent as a string
            }
        };

        console.log('Sending WsAskMedia', message);
        win.ws.send(JSON.stringify(message));
    } else {
        console.error("WebSocket is not open. Unable to send request.");
    }
}

function askFileContent(clientId: string, serverId: string, fileName: string): void {
    //Chen ask file to controller (file name with .extension given)
    if (win.ws && win.ws.readyState === WebSocket.OPEN) {
        // Construct the message with the actual values of whichClient and whichServer
        const message = {
            WsAskFileContent: {
                client_id: clientId.toString(), // Ensure u64 is sent as a string
                server_id: serverId.toString(), // Ensure u64 is sent as a string
                file_ref: fileName,
            }
        };
        win.ws.send(JSON.stringify(message));
    } else {
        console.error('WebSocket is not open. Unable to send update command.');
    }
}

function askMedia(clientId: string, reference_media: string): void {
    //Chen ask media to controller
    if (win.ws && win.ws.readyState === WebSocket.OPEN) {
        // Construct the message with the actual values of whichClient and whichServer
        const message = {
            WsAskMedia: {
                client_id: clientId.toString(), // Ensure u64 is sent as a string
                media_ref: reference_media,
            }
        };
        win.ws.send(JSON.stringify(message));
    } else {
        console.error('WebSocket is not open. Unable to send update command.');
    }
}

/*
// INITIALIZE UI WITH FIRST SERVER
document.addEventListener("DOMContentLoaded", updateServerDisplay);
*/

// FUNCTION TO UPDATE FILE LIST BASED ON SERVER
function updateFileList(files: FileList): void {
    // 'files' is expected to be an array of file names.
    const fileListTable = document.querySelector(".file-list") as HTMLElement;
    if (!fileListTable) return;

    fileListTable.innerHTML = ""; // Clear existing content

    // Get the current server ID from the sorted array.
    const currentServerId = get_server_id_from_current_server_index();

    // Update the file list for the current server.
    if (file_lists[currentServerId]) {
        file_lists[currentServerId].files = files;
    } else {
        console.error("No server found for ID", currentServerId);
    }

    if (Object.keys(files).length !== 0) {
        // Populate the new file list filtering by the currentFilterType and currentSearchValue.
        for (const [fileName, key] of Object.entries(files)) {
            if (currentFilterType) {
                const extension = fileName.split('.').pop()?.toLowerCase();
                if (extension !== currentFilterType.toLowerCase()) {
                    continue;  // Skip files that don't match the filter.
                }
            }

            // Check if the file name contains the search term.
            if (currentSearchValue) {
                if (!fileName.toLowerCase().includes(currentSearchValue.toLowerCase())) {
                    continue; // Skip if the file name doesn't match.
                }
            }

            const row = document.createElement("tr");
            row.innerHTML = `
               <td>${fileName}</td>
               <td style="text-align:right;">${file_lists[get_server_id_from_current_server_index()]?.name || ''}</td>
            `;
            // When the row is clicked, open the popup with that file.
            row.addEventListener("click", () => openPopup(fileName));
            fileListTable.appendChild(row);
        }
    }

    // Stop and hide the loading popup.
    const loadingPopup = document.getElementById("loading-popup") as HTMLElement;
    if (loadingPopup && loadingPopup.style.display === "flex") {
        loadingPopup.style.display = "none";
    }
}

function updateFile(file_content: string): void {
    // Get the file name from the popup title.
    const fileNameElement = document.getElementById("file-popup-title");
    if (!fileNameElement) return;
    
    const fileName = fileNameElement.textContent?.trim();
    if (!fileName) return;

    // Get the current server from file_lists.
    const currentServerId = get_server_id_from_current_server_index();
    const currentServer = file_lists[currentServerId];

    // Update the file content if this file exists.
    if (currentServer && currentServer.files && currentServer.files.hasOwnProperty(fileName)) {
        currentServer.files[fileName] = file_content;
    }

    // Update the popup with the new content.
    const popupFileContent = document.getElementById("file-popup-file-content") as HTMLElement;
    if (!popupFileContent) return;

    const extension = fileName.split('.').pop()?.toLowerCase();
    if (extension === "html") {
        const parts = file_content.split(/(#Media\[[^\]]*\])/g);
        const processedContent = parts.map(part => {
            const mediaMatch = part.match(/^#Media\[(.*?)\]$/);
            if (mediaMatch) {
                const reference = mediaMatch[1];
                const found = media.find(item => item.reference === reference);
                if (found && found.media) {
                    // Media is already loaded.
                    return `<img src="${found.media}" id="reference-${reference}" alt="Media loaded" />`;
                } else if (!win.requestedMedia?.has(reference)) {
                    // Request the media only if it hasn't been requested yet.
                    if (!win.requestedMedia) win.requestedMedia = new Set();
                    win.requestedMedia.add(reference); // Mark as requested
                    askMedia(win.currentClientId, reference); // Request the media
                    return `<img src="content_objects/reload.png" class="loading_image" id="reference-${reference}" alt="Loading..." />`;
                } else {
                    // Media has already been requested but is not yet loaded.
                    return `<img src="content_objects/reload.png" class="loading_image" id="reference-${reference}" alt="Loading..." />`;
                }
            } else {
                return part.trim() ? `<p>${part.trim()}</p>` : "";
            }
        }).join("");
        popupFileContent.innerHTML = processedContent;
    } else {
        popupFileContent.textContent = file_content;
    }
}

function updateMedia(mediaRef: MediaRef): void {
    console.log(mediaRef);
    const fullPath = window.location.pathname;
    // Remove the filename (assumes a filename is present)
    const basePath = fullPath.substring(0, fullPath.lastIndexOf('/'));
    // Combine with the protocol
    const absolutePath = window.location.protocol + basePath;

    for (const key in mediaRef) {
        const reference = key;
        const base64Image = mediaRef[reference];
        // Add the media to the media array.
        const existingMedia = media.find(item => item.reference === reference);
        if (!existingMedia) {
            media.push({ reference, media: base64Image });
        } else {
            existingMedia.media = base64Image;
        }
        // Find the image element with the corresponding id.
        const imgElem = document.getElementById("reference-" + reference) as HTMLImageElement;
        if (imgElem) {
            //console.log(base64Image);
            imgElem.classList.remove("loading", "rotate");  // Remove any rotation classes
            imgElem.style.animation = "none";  // Stop rotation
            imgElem.style.transform = "none";  // Reset any transforms
            imgElem.style.width = "400px";
            imgElem.style.height = "auto";
            imgElem.src = absolutePath + base64Image + "?t=" + new Date().getTime();
        } else {
            //console.warn("No element found with id:", "reference-" + reference);
        }
    }
}
