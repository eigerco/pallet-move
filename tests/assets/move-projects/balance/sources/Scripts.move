script {
    use substrate::balance;

    fun verify_preconfigured_balance(account: address, preconfigured_amount: u128) {
        let total_amount = balance::total_amount(account);

        assert!(total_amount == preconfigured_amount, 0);
    }
}
