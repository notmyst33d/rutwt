export async function upload(args) {
    for (const file of args.event.target.files) {
        const formData = new FormData();
        let fileType = "photo";
        if (file.type === "image/jpeg") {
            if (args.profilePicture !== undefined && args.profilePicture) {
                formData.append("type", "profile_picture");
                fileType = "profile_picture";
            } else if (args.banner !== undefined && args.banner) {
                formData.append("type", "banner");
                fileType = "banner";
            } else {
                formData.append("type", "photo");
            }
        } else if (file.type === "video/mp4") {
            formData.append("type", "video");
            fileType = "video";
        } else if (file.type === "audio/mpeg") {
            formData.append("type", "audio");
            fileType = "audio";
        } else {
            continue;
        }
        formData.append("data", file);
        const response = await fetch("/api/media/upload", {
            method: "POST",
            body: formData,
            headers: {
                Authorization: `Bearer ${window.localStorage.getItem("token")}`,
            },
        });
        const data = await response.json();
        if (args.onProcessingStart !== undefined) {
            args.onProcessingStart({ id: data.id, type: fileType });
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
                args.onProcessingEnd({ id: data.id, type: fileType });
                clearInterval(poll);
            }
        }, 1000);
    }
}