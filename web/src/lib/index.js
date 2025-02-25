export function upload(args) {
    const file = args.file;
    const formData = new FormData();
    let fileType = "photo";
    if (file.type === "image/jpeg" || file.type === "image/png" || file.type === "image/webp") {
        if (args.profilePicture !== undefined && args.profilePicture) {
            formData.append("type", "profile_picture");
            fileType = "profile_picture";
        } else if (args.banner !== undefined && args.banner) {
            formData.append("type", "banner");
            fileType = "banner";
        } else {
            formData.append("type", "photo");
        }
    } else if (file.type === "video/mp4" || file.type === "video/webm" || file.type === "video/x-matroska") {
        formData.append("type", "video");
        fileType = "video";
    } else if (file.type === "audio/mpeg" || file.type === "audio/mp4" || file.type === "audio/ogg") {
        formData.append("type", "audio");
        fileType = "audio";
    } else {
        alert("Неизвестный тип медиа");
        return;
    }
    formData.append("data", file);
    const request = new XMLHttpRequest();
    request.open("POST", "/api/media/upload", true);

    request.upload.addEventListener("progress", (e) => {
        if (args.onUploadProgress !== undefined && args.id !== undefined) {
            args.onUploadProgress({ id: args.id, progress: e.loaded / e.total });
        }
    });
    request.addEventListener("error", () => {
        if (args.onUploadError !== undefined && args.id !== undefined) {
            args.onUploadError({ id: args.id, error: "Соединение прервано" });
        }
    });
    request.addEventListener("load", async (e) => {
        if (request.status !== 200 && args.onUploadError !== undefined && args.id !== undefined) {
            args.onUploadError({ id: args.id, error: "Ошибка на стороне сервера" });
            return;
        }
        const data = JSON.parse(request.responseText);
        if (args.onProcessingStart !== undefined) {
            args.onProcessingStart({ id: data.id, type: fileType, error: null });
        }
        const poll = setInterval(async () => {
            const checkResponse = await fetch(
                `/api/media/check/${data.id}`,
                {
                    headers: {
                        Authorization: `Bearer ${window.localStorage.getItem("token")}`,
                    },
                },
            );
            const checkData = await checkResponse.json();
            if (!checkData.processing) {
                args.onProcessingEnd({ id: data.id, type: fileType, error: checkData.processing_error });
                clearInterval(poll);
            }
        }, 1000);
    });

    request.setRequestHeader("Authorization", `Bearer ${window.localStorage.getItem("token")}`);
    request.send(formData);
}
