<script>
    import { goto, invalidate, invalidateAll } from "$app/navigation";
    import { page } from "$app/state";
    import { upload } from "$lib";

    let { commentPostId } = $props();
    let message = $state();
    let media = $state([]);
</script>

<div class="column align-self-center post-form-container">
    <textarea
        class="align-self-center generic-textarea"
        placeholder="Напишите что у вас происходит"
        bind:value={message}
    ></textarea>
    <div class="modal-button-container">
        <!-- svelte-ignore a11y_consider_explicit_label -->
        <button
            class="button modal-button"
            onclick={() => {
                const filePicker = document.createElement("input");
                filePicker.type = "file";
                filePicker.multiple = true;
                filePicker.addEventListener("change", (e) => {
                    for (const file of e.target.files) {
                        if (media.length >= 5) {
                            alert(
                                "Можно иметь максимум 5 медиа файлов на посте.",
                            );
                            return;
                        }
                    }
                    upload({
                        event: e,
                        onProcessingStart: (m) =>
                            media.push({
                                id: m.id,
                                type: m.type,
                                processing: true,
                            }),
                        onProcessingEnd: (m) => {
                            if (m.error !== undefined && m.error !== null) {
                                alert("Произошла ошибка при обработке файла");
                                media = media.filter((mf) => mf.id !== m.id);
                                return;
                            }
                            media = media.map((mf) => {
                                if (mf.id === m.id) {
                                    mf.processing = false;
                                }
                                return mf;
                            });
                        },
                    });
                });
                filePicker.accept =
                    "image/jpeg,image/png,image/webp,audio/mpeg,audio/mp4,audio/ogg,video/mp4,video/webm,video/x-matroska";
                filePicker.click();
            }}
            ><svg
                xmlns="http://www.w3.org/2000/svg"
                height="24px"
                viewBox="0 -960 960 960"
                width="24px"
                fill="currentColor"
                ><path
                    d="M720-330q0 104-73 177T470-80q-104 0-177-73t-73-177v-370q0-75 52.5-127.5T400-880q75 0 127.5 52.5T580-700v350q0 46-32 78t-78 32q-46 0-78-32t-32-78v-330q0-17 11.5-28.5T400-720q17 0 28.5 11.5T440-680v330q0 13 8.5 21.5T470-320q13 0 21.5-8.5T500-350v-350q-1-42-29.5-71T400-800q-42 0-71 29t-29 71v370q-1 71 49 120.5T470-160q70 0 119-49.5T640-330v-350q0-17 11.5-28.5T680-720q17 0 28.5 11.5T720-680v350Z"
                /></svg
            ></button
        >
        <!-- svelte-ignore a11y_consider_explicit_label -->
        <button
            class="button modal-button"
            onclick={async () => {
                const mediaIds = [];
                for (const mediaEntry of media) {
                    if (mediaEntry.processing) return;
                    mediaIds.push(mediaEntry.id);
                }
                const body = {
                    message,
                    media: mediaIds,
                };
                if (commentPostId !== undefined) {
                    body.comment_post_id = commentPostId;
                }
                const response = await fetch("/api/posts/create", {
                    method: "POST",
                    body: JSON.stringify(body),
                    headers: {
                        "Content-Type": "application/json",
                        Authorization: `Bearer ${window.localStorage.getItem("token")}`,
                    },
                });
                if (response.status !== 200) {
                    alert("Произошла ошибка при создании поста");
                } else {
                    const data = await response.json();
                    if (commentPostId !== undefined) {
                        invalidate("data:comments");
                        message = "";
                        media = [];
                    } else {
                        goto(`/${page.data.user.username}/status/${data.id}`);
                    }
                }
            }}
            ><svg
                xmlns="http://www.w3.org/2000/svg"
                height="24px"
                viewBox="0 -960 960 960"
                width="24px"
                fill="currentColor"
                ><path
                    d="M792-443 176-183q-20 8-38-3.5T120-220v-520q0-22 18-33.5t38-3.5l616 260q25 11 25 37t-25 37ZM200-280l474-200-474-200v140l240 60-240 60v140Zm0 0v-400 400Z"
                /></svg
            ></button
        >
    </div>
    <div class="row post-form-media">
        {#each media as mediaEntry}
            <div class="post-form-media-container">
                {#if mediaEntry.processing}
                    <div class="loader"></div>
                {:else if mediaEntry.type === "photo" || mediaEntry.type === "video"}
                    <!-- svelte-ignore a11y_missing_attribute -->
                    <img
                        src="/api/media/{mediaEntry.id}.jpg:{mediaEntry.type ==
                        'photo'
                            ? 'small'
                            : 'thumbnail'}"
                    />
                {:else if mediaEntry.type === "audio"}
                    <div class="music-note">
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            height="32px"
                            viewBox="0 -960 960 960"
                            width="32px"
                            fill="currentColor"
                            ><path
                                d="M400-120q-66 0-113-47t-47-113q0-66 47-113t113-47q23 0 42.5 5.5T480-418v-382q0-17 11.5-28.5T520-840h160q17 0 28.5 11.5T720-800v80q0 17-11.5 28.5T680-680H560v400q0 66-47 113t-113 47Z"
                            /></svg
                        >
                    </div>
                {/if}
                {#if !mediaEntry.processing}
                    <!-- svelte-ignore a11y_consider_explicit_label -->
                    <button
                        class="button post-form-remove-button"
                        onclick={() =>
                            (media = media.filter(
                                (e) => e.id != mediaEntry.id,
                            ))}
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            height="24px"
                            viewBox="0 -960 960 960"
                            width="24px"
                            fill="currentColor"
                            ><path
                                d="M480-424 284-228q-11 11-28 11t-28-11q-11-11-11-28t11-28l196-196-196-196q-11-11-11-28t11-28q11-11 28-11t28 11l196 196 196-196q11-11 28-11t28 11q11 11 11 28t-11 28L536-480l196 196q11 11 11 28t-11 28q-11 11-28 11t-28-11L480-424Z"
                            />
                        </svg>
                    </button>
                {/if}
            </div>
        {/each}
    </div>
</div>

<style>
    .music-note {
        margin-left: 16px;
        margin-top: 16px;
    }

    .loader {
        width: 6px;
        margin: auto;
        margin-top: 26px;
        aspect-ratio: 1;
        border-radius: 50%;
        animation: l5 1s infinite linear alternate;
    }
    @keyframes l5 {
        0% {
            box-shadow:
                10px 0 #ffffff,
                -10px 0 rgba(255, 255, 255, 0.133);
            background: #ffffff;
        }
        33% {
            box-shadow:
                10px 0 #ffffff,
                -10px 0 rgba(255, 255, 255, 0.133);
            background: rgba(255, 255, 255, 0.133);
        }
        66% {
            box-shadow:
                10px 0 rgba(255, 255, 255, 0.133),
                -10px 0 #ffffff;
            background: rgba(255, 255, 255, 0.133);
        }
        100% {
            box-shadow:
                10px 0 rgba(255, 255, 255, 0.133),
                -10px 0 #ffffff;
            background: #ffffff;
        }
    }
</style>
