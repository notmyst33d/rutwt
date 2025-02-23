<script>
    import { goto, invalidateAll } from "$app/navigation";
    import { page } from "$app/state";
    import { upload } from "$lib";
    import Navbar from "$lib/Navbar.svelte";
    import Post from "$lib/Post.svelte";

    let bannerPhotoId = $state(page.data.user.banner_photo_id);
    let profilePicturePhotoId = $state(page.data.user.profile_picture_photo_id);
    let realname = $state(page.data.user.realname);
    let username = $state(page.data.user.username);
    let bio = $state(page.data.user.bio);
</script>

<div class="column">
    <Navbar />
    <div class="column feed-container">
        <div class="column profile margin-bottom-8">
            <div class="settings-banner-container">
                {#if bannerPhotoId !== null}
                    <!-- svelte-ignore a11y_missing_attribute -->
                    <img class="header" src="/api/media/{bannerPhotoId}.jpg" />
                {:else}
                    <div class="header gray-bg"></div>
                {/if}
                <!-- svelte-ignore a11y_consider_explicit_label -->
                <button
                    class="button settings-banner-change-button"
                    onclick={() => {
                        const filePicker = document.createElement("input");
                        filePicker.type = "file";
                        filePicker.multiple = true;
                        filePicker.addEventListener("change", (e) =>
                            upload({
                                event: e,
                                banner: true,
                                onProcessingEnd: (m) =>
                                    (bannerPhotoId = m.id),
                            }),
                        );
                        filePicker.accept = "image/jpeg";
                        filePicker.click();
                    }}
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        height="24px"
                        viewBox="0 -960 960 960"
                        width="24px"
                        fill="currentColor"
                        ><path
                            d="M200-200h57l391-391-57-57-391 391v57Zm-80 80v-170l528-527q12-11 26.5-17t30.5-6q16 0 31 6t26 18l55 56q12 11 17.5 26t5.5 30q0 16-5.5 30.5T817-647L290-120H120Zm640-584-56-56 56 56Zm-141 85-28-29 57 57-29-28Z"
                        /></svg
                    >
                </button>
            </div>
            <!-- svelte-ignore a11y_missing_attribute -->
            <div class="row wrap settings-container">
                <div
                    class="settings-profile-picture-container height-fit-content"
                >
                    <img
                        class="big-profile-picture"
                        src={profilePicturePhotoId !== null
                            ? `/api/media/${profilePicturePhotoId}.jpg`
                            : "/placeholder/nopfp.png"}
                    />
                    <!-- svelte-ignore a11y_consider_explicit_label -->
                    <button
                        class="button settings-profile-picture-change-button"
                        onclick={() => {
                            const filePicker = document.createElement("input");
                            filePicker.type = "file";
                            filePicker.multiple = true;
                            filePicker.addEventListener("change", (e) =>
                                upload({
                                    event: e,
                                    profilePicture: true,
                                    onProcessingEnd: (m) =>
                                        (profilePicturePhotoId = m.id),
                                }),
                            );
                            filePicker.accept = "image/jpeg";
                            filePicker.click();
                        }}
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            height="24px"
                            viewBox="0 -960 960 960"
                            width="24px"
                            fill="currentColor"
                            ><path
                                d="M200-200h57l391-391-57-57-391 391v57Zm-80 80v-170l528-527q12-11 26.5-17t30.5-6q16 0 31 6t26 18l55 56q12 11 17.5 26t5.5 30q0 16-5.5 30.5T817-647L290-120H120Zm640-584-56-56 56 56Zm-141 85-28-29 57 57-29-28Z"
                            /></svg
                        >
                    </button>
                </div>
                <!-- svelte-ignore a11y_label_has_associated_control -->
                <div class="column">
                    <!-- svelte-ignore a11y_label_has_associated_control -->
                    <label>Имя</label>
                    <input
                        type="text"
                        class="text-box field"
                        placeholder="Имя"
                        bind:value={realname}
                    />
                    <label class="margin-top-8">Юзернейм</label>
                    <input
                        type="text"
                        class="text-box field"
                        placeholder="Юзернейм"
                        bind:value={username}
                    />
                </div>
                <!-- svelte-ignore a11y_label_has_associated_control -->
                <div class="column grow">
                    <label>Био</label>
                    <textarea
                        class="generic-textarea"
                        placeholder="Био"
                        bind:value={bio}
                    ></textarea>
                </div>
            </div>
            <button
                class="button margin-top-8"
                onclick={async () => {
                    const response = await fetch("/api/users/settings", {
                        method: "POST",
                        body: JSON.stringify({
                            realname,
                            username,
                            bio,
                            profile_picture_photo_id: profilePicturePhotoId,
                            banner_photo_id: bannerPhotoId,
                        }),
                        headers: {
                            "Content-Type": "application/json",
                            Authorization: `Bearer ${window.localStorage.getItem("token")}`,
                        },
                    });
                    goto(`/${username}`, { invalidateAll: true });
                }}>Применить</button
            >
        </div>
    </div>
</div>

<style>
    .settings-container {
        margin-top: 8px;
        gap: 8px;
    }
</style>
