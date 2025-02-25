<script>
    let { additionalQueryParams, onLoad, noMoreData = $bindable() } = $props();
    let offset = $state(0);
    let loadInProgress = $state(false);
</script>

<svelte:window
    onscroll={async (e) => {
        let pixelsToBottom = Math.abs(
            window.scrollY + window.innerHeight - document.body.scrollHeight,
        );
        if (pixelsToBottom < 500 && !loadInProgress && !noMoreData) {
            loadInProgress = true;
            offset += 100;
            const response = await fetch(
                `/api/posts/find?offset=${offset}${additionalQueryParams ? additionalQueryParams : ""}`,
                {
                    headers: {
                        Authorization: `Bearer ${window.localStorage.getItem("token")}`,
                    },
                },
            );
            if (response.status === 200) {
                const data = await response.json();
                if (data.length === 0) {
                    noMoreData = true;
                    onNoMoreData();
                } else {
                    onLoad(data);
                }
            } else {
                alert("Загрузка новых постов не удалась");
                noMoreData = true;
            }
            loadInProgress = false;
        }
    }}
/>
