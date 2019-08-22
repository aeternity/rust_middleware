// import Vue from 'vue'

export const state = () => ({
  generations: [],
  transactions: [],
  socket: null,
  connected: false,
  subscribed: false
})

export const mutations = {
  setWs: function (state, socket) {
    state.socket = socket
  },
  setConnectionStatus: function (state, status) {
    state.connected = status
  },
  setGenerations: function (state, generation) {
    generation['micro_blocks'] = {}
    const gen = state.generations.find((el) => {
      return el.height === generation.height
    })
    if (!gen) {
      if (state.generations.length === 0 || (generation.height > state.generations[0].height)) {
        state.generations.unshift(generation)
        if (state.generations.length >= 5) {
          state.generations.splice(5)
        }
      }
    }
  },
  setSubscribed: function (state, status) {
    state.subscribed = status
  },
  setTransactions: function (state, transaction) {
    state.transactions.unshift(transaction)
    if (state.transactions.length >= 5) {
      state.transactions.splice(5)
    }
  },
  updateGenerationTx: function (state, tx) {
    const gen = state.generations.find((el) => {
      return el.height === tx.block_height
    })
    if (gen) {
      if (!gen.micro_blocks[tx.block_hash]) {
        gen.micro_blocks[tx.block_hash] = { transactions: {} }
      }
      gen.micro_blocks[tx.block_hash].transactions[tx.hash] = tx
      const index = state.generations.findIndex(el => el.height === gen.height)
      state.generations[index] = gen
    }
  }
}

export const actions = {

  initializeWs: function ({ rootState: { wsUrl }, commit, dispatch }) {
    const socket = new WebSocket(wsUrl)
    commit('setWs', socket)
    socket.onopen = () => {
      commit('setConnectionStatus', true)
      dispatch('subscribeEvents')
    }
  },

  closeConnection: function ({ state, commit }) {
    if (state.socket) {
      state.socket.close()
      commit('setWs', null)
      commit('setConnectionStatus', false)
    }
  },

  subscribeEvents: function ({ state, commit, dispatch }) {
    state.socket.send('{"op":"subscribe", "payload": "key_blocks"}')
    state.socket.send('{"op":"subscribe", "payload": "transactions"}')
    commit('subscribed', true)
    state.socket.onmessage = e => {
      dispatch('processWsMessage', e.data)
    }
  },

  processWsMessage: function ({ commit }, data) {
    if (data.includes('payload')) {
      data = JSON.parse(data).payload
      if (data.tx) {
        commit('setTransactions', data)
        commit('updateGenerationTx', data)
      } else if (data.beneficiary) {
        commit('setGenerations', data)
      }
    }
  },
  nuxtServerInit ({ dispatch, commit }, context) {
    const transactions = Object.values(context.transactions)
    transactions.splice(0, 5)
    const generations = Object.values(context.generations)
    generations.splice(0, 5)
    for (let i = 0; i < 5; i++) {
      commit('setGenerations', generations[i])
      commit('setTransactions', transactions[i])
    }
  }
}
