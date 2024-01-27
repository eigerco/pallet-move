script {
    use DeveloperBob::CarWash;
    
    fun initial_coin_minting(account: signer) {
        CarWash::initial_coin_minting(&account);
    }
}

script {
    use DeveloperBob::CarWash;
    
    fun register_new_user(account: signer) {
        CarWash::register_new_user(&account);
    }
}

script {
    use DeveloperBob::CarWash;
    
    fun buy_coin(account: signer) {
        CarWash::buy_coin(&account);
    }
}

script {
    use DeveloperBob::CarWash;
    
    fun wash_car(account: signer) {
        CarWash::wash_car(&account);
    }
}
