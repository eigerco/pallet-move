script {
    use DeveloperBob::Counter;

    fun count(account: signer) {
        // TODO, use return value somehow
        Counter::count(&account);
    }
}
