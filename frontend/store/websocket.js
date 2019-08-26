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

  processWsMessage: function ({ rootState: { height }, commit, dispatch }, data) {
    if (data.includes('payload')) {
      data = JSON.parse(data).payload
      if (data.tx) {
        commit('setTransactions', data)
      } else if (data.beneficiary) {
        commit('setGenerations', data)
        if (data.height > height) {
          dispatch('height')
        }
      }
    }
  },
  nuxtServerInit ({ dispatch, commit }, context) {
    Object.values(context.transactions).forEach(element => {
      commit('setTransactions', element)
    })
    Object.values(context.generations).forEach(element => {
      commit('setGenerations', element)
    })
  }
}
