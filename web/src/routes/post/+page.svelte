<script>
    import { goto } from "$app/navigation";
    import { page } from "$app/state";
    import Navbar from "$lib/Navbar.svelte";
    import PostForm from "$lib/PostForm.svelte";

    let message;
</script>

<div class="column">
    <Navbar />
    <div class="column">
        <span class="action align-self-center">Новый пост</span>
        <PostForm />
    </div>
</div>
