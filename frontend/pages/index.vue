<template>
  <div>
    <no-ssr>
      <div
        v-if="generations.length"
        class="generations-wrapper"
      >
        <PageHeader
          :is-main="false"
          title="Generations"
        />
        <Generations>
          <Generation
            v-for="(generation, number) in generations"
            :key="number"
            class="generation-link"
            :data="generation"
          />
        </Generations>
      </div>
      <div
        v-if="transactions.length"
        class="transactions-wrapper"
      >
        <PageHeader
          :is-main="false"
          title="Transactions"
        />
        <TxList>
          <TXListItem
            v-for="(transaction, index) in transactions"
            :key="index"
            :data="transaction"
          />
        </TxList>
      </div>
    </no-ssr>
  </div>
</template>
<script>
import Generations from '../partials/generations'
import Generation from '../partials/generation'
import TxList from '../partials/transactions/txList'
import TXListItem from '../partials/transactions/txListItem'
import PageHeader from '../components/PageHeader'
import { mapState } from 'vuex'

export default {
  name: 'AppDashboard',
  layout: 'default',
  components: {
    Generations,
    Generation,
    TxList,
    TXListItem,
    PageHeader
  },
  computed: {
    ...mapState('websocket', {
      generations (state) {
        return state.generations
      }
    }),
    ...mapState('websocket', {
      transactions (state) {
        return state.transactions
      }
    })
  },
  async mounted () {
    this.$store.dispatch('websocket/initializeWs')
    if (!Object.keys(this.$store.state.generations.generations).length) {
      await this.$store.dispatch('height')
      this.$store.dispatch('generations/getLatestGenerations', 10)
    }
    if (!Object.keys(this.$store.state.transactions.transactions).length) {
      await this.$store.dispatch('height')
      this.$store.dispatch('transactions/getLatestTransactions', { limit: 10 })
    }
  }
}
</script>
