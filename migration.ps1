For ($i = 0; $i -lt 10; $i++) {
    diesel migration revert
}
diesel migration run
